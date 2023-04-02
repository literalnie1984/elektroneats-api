use actix_web::{get, post, web, HttpRequest};
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::{borrow::Borrow, collections::HashMap, mem};
use stripe::{
    self, CreateCustomer, CreatePaymentIntent, Customer, EventObject, EventType, PaymentIntent,
    UpdateCustomer, Webhook,
};

use crate::{
    appstate::AppState, convert_err_to_500, errors::ServiceError, get_header_val, get_user,
    jwt_auth::AuthUser, map_db_err,
};

use super::structs::{AddReturn, StripeUser};

#[post("/add-balance/{amount:[0-9]+}")]
async fn add_balance(
    data: web::Data<AppState>,
    amount: web::Path<i64>,
    user: AuthUser,
) -> Result<web::Json<AddReturn>, ServiceError> {
    let client = &data.stripe_client.0;
    let customer = get_user(&data.conn, user.id, client).await?;
    let customer_id = customer.id.to_string();

    let intent = {
        let mut intent = CreatePaymentIntent::new(amount.into_inner(), stripe::Currency::PLN);
        intent.payment_method_types = Some(vec!["card".into(), "p24".into()]);
        intent.customer = Some(customer.id);
        intent.expand = &["customer"];

        PaymentIntent::create(client, intent)
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe Error")))?
    };

    Ok(web::Json(AddReturn {
        customer_id,
        intent_secret: intent.client_secret.unwrap(),
    }))
}

//Webhook for stripe to use
#[post("/received")]
async fn received_payment(
    req: HttpRequest,
    payload: web::Bytes,
    data: web::Data<AppState>,
) -> Result<String, ServiceError> {
    let payload_str = std::str::from_utf8(payload.borrow()).unwrap();

    let stripe_sig = get_header_val(&req, "stripe-signature").unwrap_or_default();

    let web_hook_secret: String =
        dotenvy::var("WEBHOOK_SECRET").expect("No WEBHOOK_SECRET provided in .env");
    let Ok(event) = Webhook::construct_event(payload_str, stripe_sig, &web_hook_secret) else {return Err(ServiceError::InternalError)};

    //for dev reasons @ release switch to payment intent success
    if event.event_type == EventType::PaymentIntentSucceeded {
        let EventObject::PaymentIntent(intent_data) = event.data.object else {return Err(ServiceError::InternalError)};

        let client = &data.stripe_client.0;
        let customer = &intent_data.customer.unwrap();
        let customer_id = &customer.id();
        let customer_balance = Customer::retrieve(client, customer_id, &[])
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?
            .balance
            .unwrap();

        Customer::update(
            client,
            customer_id,
            UpdateCustomer {
                balance: Some(intent_data.amount + customer_balance),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?;
    } else {
        return Err(ServiceError::InternalError);
    }

    Ok("tak".into())
}

#[get("/balance")]
async fn get_balance(user: AuthUser, data: web::Data<AppState>) -> Result<String, ServiceError> {
    let user = get_user(&data.conn, user.id, &data.stripe_client.0).await?;
    Ok(serde_json::json!({"balance": user.balance.unwrap()}).to_string())
}

#[get("/details")]
async fn customer_details(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<Customer>, ServiceError> {
    let user = get_user(&data.conn, user.id, &data.stripe_client.0).await?;
    Ok(web::Json(user))
}

#[post("/init")]
async fn init_wallet(
    user: AuthUser,
    data: web::Data<AppState>,
    mut stripe_data: web::Json<StripeUser>,
) -> Result<String, ServiceError> {
    if !user.is_verified {
        return Err(ServiceError::BadRequest(
            "Your account must be validated before initializing wallet".into(),
        ));
    }

    let client = &data.stripe_client.0;
    let conn = &data.conn;

    let user = User::find_by_id(user.id)
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if user.stripe_id.is_some() {
        return Err(ServiceError::BadRequest(
            "Provided user is already registered in stripe".into(),
        ));
    }

    let addr = {
        let addr = &mut stripe_data.address;
        stripe::Address {
            city: Some(mem::take(&mut addr.city)),
            country: Some(mem::take(&mut addr.country)),
            postal_code: Some(mem::take(&mut addr.postal_code)),
            state: Some(mem::take(&mut addr.state)),
            ..Default::default()
        }
    };
    let customer = Customer::create(
        client,
        CreateCustomer {
            email: Some(&user.email),
            name: Some(&stripe_data.name),
            phone: Some(&stripe_data.phone),
            address: Some(addr),
            metadata: Some(HashMap::from([("async-stripe".into(), "true".into())])),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?;

    let mut user_upd: user::ActiveModel = user.into();
    user_upd.stripe_id = Set(Some(customer.id.to_string()));
    user_upd.update(conn).await.map_err(map_db_err)?;

    Ok("Success".into())
}

// USE add_balance for testing - it works too!
/*
//test path for user with id 1
#[get("/test_balance")]
async fn test_balance(data: web::Data<AppState>) -> Result<String, ServiceError> {
    let client = &data.stripe_client.0;
    let customer = get_user(&data.conn, 1, client).await?;

    let intent = {
        let mut intent = CreatePaymentIntent::new(5000, stripe::Currency::PLN);
        intent.payment_method_types = Some(vec!["card".into(), "p24".into()]);
        intent.customer = Some(customer.id.clone());
        intent.expand = &["customer"];

        PaymentIntent::create(client, intent)
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe Error")))?
    };

    let payment_method = {
        let pm = PaymentMethod::create(
            client,
            CreatePaymentMethod {
                type_: Some(PaymentMethodTypeFilter::Card),
                card: Some(stripe::CreatePaymentMethodCardUnion::CardDetailsParams(
                    CardDetailsParams {
                        number: "4000006160000005".to_string(), // UK visa
                        exp_year: 2025,
                        exp_month: 1,
                        cvc: Some("123".to_string()),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        pm
    };

    PaymentMethod::attach(
        client,
        &payment_method.id,
        AttachPaymentMethod {
            customer: customer.id.clone(),
        },
    );

    let intent = PaymentIntent::update(
        client,
        &intent.id,
        UpdatePaymentIntent {
            payment_method: Some(payment_method.id),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    PaymentIntent::confirm(
        client,
        &intent.id,
        PaymentIntentConfirmParams {
            ..Default::default()
        },
    );

    Ok("send".into())
}
*/
