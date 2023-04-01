use std::{borrow::Borrow, collections::HashMap, str::FromStr};

use actix_web::{get, post, web, HttpRequest};
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Serialize;
use std::mem;
use stripe::{
    self, AttachPaymentMethod, CardDetailsParams, Client, CreateCustomer, CreatePaymentIntent,
    CreatePaymentMethod, Customer, CustomerId, EventObject, EventType, PaymentIntent,
    PaymentIntentConfirmParams, PaymentMethod, PaymentMethodTypeFilter, UpdateCustomer,
    UpdatePaymentIntent, Webhook,
};

use crate::{
    appstate::AppState, convert_err_to_500, errors::ServiceError, get_header_val,
    jwt_auth::AuthUser, map_db_err,
};

use super::structs::{AddReturn, StripeUser};

//for development purposes
const WEB_HOOK_SECRET: &'static str =
    "whsec_f8729a0c2875d30a6466924337e8af2057a894595d348f0e0092878ec1e40d08";

#[post("/add_balance/{amount:[0-9]+}")]
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

    let Ok(event) = Webhook::construct_event(payload_str, stripe_sig, WEB_HOOK_SECRET) else {return Err(ServiceError::InternalError)};

    //for dev reasons @ release switch to payment intent success
    if event.event_type == EventType::PaymentIntentCreated {
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

#[post("/pay/{amount:[0-9]+}")]
async fn pay() -> Result<String, ServiceError> {
    todo!()
}

#[derive(Serialize)]
struct Balance {
    balance: i64,
}

#[get("/balance")]
async fn get_balance(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<Balance>, ServiceError> {
    let user = get_user(&data.conn, user.id, &data.stripe_client.0).await?;
    Ok(web::Json(Balance {
        balance: user.balance.unwrap(),
    })) // 1564 -> 15.64
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

async fn get_user(
    conn: &DatabaseConnection,
    user_id: i32,
    client: &Client,
) -> Result<Customer, ServiceError> {
    let user = User::find_by_id(user_id)
        .one(conn)
        .await
        .map_err(map_db_err)?;
    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if let Some(user_id) = user.stripe_id {
        let id: CustomerId = CustomerId::from_str(&user_id).unwrap();
        let customer = Customer::retrieve(client, &id, &[])
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe err")))?;
        Ok(customer)
    } else {
        Err(ServiceError::BadRequest(
            "Stripe wasn't initialized for provided user".into(),
        ))
    }
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
