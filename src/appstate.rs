use std::fmt;
use std::{collections::HashMap, sync::Arc};
use stripe;

use async_std::sync::RwLock;
use sea_orm::DatabaseConnection;

pub type ActivatorsVec = Arc<RwLock<HashMap<String, String>>>;
#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub activators_del: ActivatorsVec,
    pub activators_reg: ActivatorsVec,
    pub stripe_client: ClientWrapper,
}

#[derive(Clone)]
pub struct ClientWrapper(pub stripe::Client);
impl fmt::Debug for ClientWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stripe client").finish()
    }
}
impl ClientWrapper {
    pub fn from_client(client: stripe::Client) -> Self {
        Self(client)
    }
    pub fn new(secret_key: &str) -> Self {
        let stripe = stripe::Client::new(secret_key);
        Self(stripe)
    }
}
