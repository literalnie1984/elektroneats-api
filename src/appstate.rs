use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sea_orm::DatabaseConnection;

pub type ActivatorsVec = Arc<RwLock<HashMap<String, String>>>;
#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub activators: ActivatorsVec,
}
