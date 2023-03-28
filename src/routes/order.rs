use actix_web::{post, web};

use crate::{jwt_auth::AuthUser, errors::ServiceError, routes::structs::OrderRequest};

#[post("/create")]
async fn create_order(/*user: AuthUser,*/ order: web::Json<OrderRequest>) -> Result<String, ServiceError>{
    let order = order.into_inner();

    Ok(format!("Order: {}, user: {:#?}", order.dinner_id, order.extras_ids))
}