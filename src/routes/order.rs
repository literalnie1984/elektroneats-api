use std::{
    collections::{HashMap, HashSet},
    mem,
};

use actix_web::{get, post, web};
use entity::{
    dinner, dinner_orders, extras, extras_order, model_enums::Status, user, user_dinner_orders,
};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{
    ActiveEnum, ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter,
    QuerySelect, Set,
};

use crate::{
    appstate::AppState,
    convert_err_to_500,
    errors::ServiceError,
    get_user,
    jwt_auth::AuthUser,
    map_db_err, pay,
    routes::structs::{
        AllUsersOrders, DinnerResponse, OrderRequest, OrderResponse, UserOrders, UserWithOrders,
    },
};

//TODO: TRANSACTIONS PROBABLY
#[post("/create")]
async fn create_order(
    user: AuthUser,
    data: web::Data<AppState>,
    order: web::Json<OrderRequest>,
) -> Result<String, ServiceError> {
    let db = &data.conn;
    let order = order.into_inner();
    let user_id = user.id;

    let dinner_ids = order
        .dinners
        .iter()
        .map(|x| x.dinner_id)
        .collect::<Vec<_>>();

    let extras_ids = order
        .dinners
        .iter()
        .flat_map(|x| x.extras_ids.clone())
        .collect::<Vec<_>>();

    let dinners: Vec<(i32, Decimal)> = dinner::Entity::find()
        .filter(dinner::Column::Id.is_in(dinner_ids.clone()))
        .select_only()
        .column(dinner::Column::Id)
        .column(dinner::Column::Price)
        .into_tuple()
        .all(db)
        .await
        .map_err(map_db_err)?;
    let extras: Vec<(i32, Decimal)> = extras::Entity::find()
        .filter(extras::Column::Id.is_in(extras_ids.clone()))
        .select_only()
        .column(extras::Column::Id)
        .column(extras::Column::Price)
        .into_tuple()
        .all(db)
        .await
        .map_err(map_db_err)?;
    let dinners: HashMap<_, _> = dinners.into_iter().collect();
    let extras: HashMap<_, _> = extras.into_iter().collect();

    let price: i64 = dinner_ids
        .into_iter()
        .map(|x| (dinners.get(&x).unwrap_or(&Decimal::ZERO).to_f64().unwrap() * 100f64) as i64)
        .sum::<i64>()
        + extras_ids
            .into_iter()
            .map(|x| (extras.get(&x).unwrap_or(&Decimal::ZERO).to_f64().unwrap() * 100f64) as i64)
            .sum::<i64>();

    let customer = get_user(db, user.id, &data.stripe_client.0).await?;
    let balance = customer.balance.unwrap();
    if balance < price {
        return Err(ServiceError::BadRequest("Not enough money".to_string()));
    }

    let dinner_order = dinner_orders::ActiveModel {
        user_id: Set(user_id),
        collection_date: Set(order.collection_date),
        status: Set(Status::Paid.into_value()),
        ..Default::default()
    };

    let dinner_order_res = dinner_orders::Entity::insert(dinner_order)
        .exec(db)
        .await
        .map_err(|e| convert_err_to_500(e, Some("Database error creating dinner_orders: {}")))?;

    for dinner in order.dinners {
        let dinner_order_junction = user_dinner_orders::ActiveModel {
            order_id: Set(dinner_order_res.last_insert_id),
            dinner_id: Set(dinner.dinner_id),
            ..Default::default()
        };

        let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction)
            .exec(db)
            .await
            .map_err(|e| {
                convert_err_to_500(e, Some("Database error creating user_dinner_orders: {}"))
            })?;

        let vector = dinner
            .extras_ids
            .into_iter()
            .map(|extra_id| extras_order::ActiveModel {
                user_dinner_id: Set(dinner_order_res.last_insert_id),
                extras_id: Set(extra_id),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        extras_order::Entity::insert_many(vector)
            .exec(db)
            .await
            .map_err(|e| convert_err_to_500(e, Some("Database error creating extras_order: {}")))?;
    }

    pay(&data.stripe_client.0, db, user, price).await?;
    Ok("Order created successfully".to_string())
}

async fn get_user_orders(
    user_id: i32,
    db: &DatabaseConnection,
    status: &[Status],
) -> Result<web::Json<UserOrders>, ServiceError> {
    let db_err = |err| convert_err_to_500(err, Some("Database error getting user orders"));
    let orders = dinner_orders::Entity::find()
        .filter(dinner_orders::Column::UserId.eq(user_id))
        .filter(dinner_orders::Column::Status.is_in(status.to_vec()))
        .all(db)
        .await
        .map_err(db_err)?;

    let user_dinner_orders = orders
        .load_many(user_dinner_orders::Entity, db)
        .await
        .map_err(db_err)?;

    let mut dinners_out = HashSet::new();
    let mut extras_out = HashSet::new();
    let mut output: Vec<OrderResponse> = Vec::new();

    for (order, user_dinner) in orders.into_iter().zip(user_dinner_orders.into_iter()) {
        let dinner = user_dinner.load_one(dinner::Entity, db).await?;

        let extras = user_dinner
            .load_many_to_many(extras::Entity, extras_order::Entity, db)
            .await
            .map_err(db_err)?;

        let mut dinners_with_extras = dinner
            .into_iter()
            .zip(extras.into_iter())
            .map(|(dinner, extras)| {
                let dinner = dinner.unwrap();
                let dinner_id = dinner.id;
                dinners_out.insert(dinner);

                let mut extras = extras
                    .into_iter()
                    .map(|e| {
                        let id = e.id;
                        extras_out.insert(e);
                        id
                    })
                    .collect::<Vec<_>>();

                DinnerResponse {
                    dinner_id,
                    extras_ids: mem::take(&mut extras),
                }
            })
            .collect::<Vec<_>>();

        output.push(OrderResponse {
            order_id: order.id,
            collection_date: order.collection_date,
            status: Status::from_repr(order.status).unwrap(),
            dinners: mem::take(&mut dinners_with_extras),
        });
    }

    Ok(web::Json(UserOrders {
        response: mem::take(&mut output),
        dinners: mem::take(&mut dinners_out),
        extras: mem::take(&mut extras_out),
    }))
}

#[get("/completed")]
async fn get_completed_user_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<UserOrders>, ServiceError> {
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(user_id, db, &[Status::Collected]).await
}

#[get("/pending")]
async fn get_pending_user_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<UserOrders>, ServiceError> {
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(
        user_id,
        db,
        &[Status::Paid, Status::Prepared, Status::Ready],
    )
    .await
}

#[get("/")]
async fn get_all_user_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<UserOrders>, ServiceError> {
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(
        user_id,
        db,
        &[
            Status::Paid,
            Status::Prepared,
            Status::Ready,
            Status::Collected,
        ],
    )
    .await
}

#[get("/")]
async fn get_all_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<AllUsersOrders>, ServiceError> {
    if !user.is_admin {
        return Err(ServiceError::Unauthorized(
            "Only admin can access that data".to_string(),
        ));
    }
    let db = &data.conn;

    let users_with_orders = user::Entity::find()
        .find_with_related(dinner_orders::Entity)
        .all(db)
        .await
        .map_err(map_db_err)?;

    get_all_orders_from_users(db, users_with_orders).await
}

#[get("/pending")]
async fn get_all_pending_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<AllUsersOrders>, ServiceError> {
    if !user.is_admin {
        return Err(ServiceError::Unauthorized(
            "Only admin can access that data".to_string(),
        ));
    }
    let db = &data.conn;

    let users_with_orders = user::Entity::find()
        .find_with_related(dinner_orders::Entity)
        .filter(dinner_orders::Column::Status.ne(Status::Collected))
        .all(db)
        .await
        .map_err(map_db_err)?;

    get_all_orders_from_users(db, users_with_orders).await
}

async fn get_all_orders_from_users(
    db: &DatabaseConnection,
    users_with_orders: Vec<(user::Model, Vec<dinner_orders::Model>)>,
) -> Result<web::Json<AllUsersOrders>, ServiceError> {
    let mut dinners_out = HashSet::new();
    let mut extras_out = HashSet::new();
    let mut output: Vec<UserWithOrders> = Vec::with_capacity(users_with_orders.len());

    for (user, orders) in users_with_orders.iter() {
        output.push(UserWithOrders {
            username: user.username.clone(),
            user_id: user.id,
            orders: Vec::new(),
        });

        let user_dinner_orders = orders
            .load_many(user_dinner_orders::Entity, db)
            .await
            .map_err(map_db_err)?;

        for (user_dinner, order) in user_dinner_orders.iter().zip(orders.iter()) {
            let dinner = user_dinner
                .load_one(dinner::Entity, db)
                .await
                .map_err(map_db_err)?;
            let extras = user_dinner
                .load_many_to_many(extras::Entity, extras_order::Entity, db)
                .await
                .map_err(map_db_err)?;

            let mut dinners_with_extras = dinner
                .into_iter()
                .zip(extras.into_iter())
                .map(|(dinner, extras)| {
                    let dinner = dinner.unwrap();
                    let dinner_id = dinner.id;
                    dinners_out.insert(dinner);

                    let mut extras = extras
                        .into_iter()
                        .map(|e| {
                            let id = e.id;
                            extras_out.insert(e);
                            id
                        })
                        .collect::<Vec<_>>();

                    DinnerResponse {
                        dinner_id,
                        extras_ids: mem::take(&mut extras),
                    }
                })
                .collect::<Vec<_>>();

            output.last_mut().unwrap().orders.push(OrderResponse {
                order_id: order.id,
                collection_date: order.collection_date,
                status: Status::from_repr(order.status).unwrap(),
                dinners: mem::take(&mut dinners_with_extras),
            });
        }
    }

    Ok(web::Json(AllUsersOrders {
        response: mem::take(&mut output),
        dinners: mem::take(&mut dinners_out),
        extras: mem::take(&mut extras_out),
    }))
}
