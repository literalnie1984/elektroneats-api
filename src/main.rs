use actix_web::{App, HttpResponse, HttpServer, Responder, web, post};
use migration::{Migrator, MigratorTrait, DbErr};
use sea_orm::{DatabaseConnection, DbConn, ActiveValue, ActiveModelTrait};

use entity::user;

struct Mutation;
impl Mutation {
    pub async fn add_user(db: &DbConn, user_data: user::Model) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            username: ActiveValue::set(user_data.username),
            password: ActiveValue::set(user_data.password),
            ..Default::default()
        }
        .save(db)
        .await
    }
}

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[post("/add")]
async fn add_user(user: web::Json<user::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    Mutation::add_user(conn, user).await.unwrap();

    HttpResponse::Ok().body("Hello world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect(".env file not found");
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    
    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    let state = AppState { conn: connection };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(add_user)
        })
        .bind(("127.0.0.1", 4765))?
        .run()
        .await
}
