use sea_orm::DatabaseConnection;

pub mod routes;
pub mod scraper;

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
}
