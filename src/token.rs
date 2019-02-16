use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    pub value: Option<String>,
    pub user: String,
    pub expires: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub value: String,
    pub user: String,
    pub expires: Option<u64>,
}

pub fn generate() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(24).collect()
}
