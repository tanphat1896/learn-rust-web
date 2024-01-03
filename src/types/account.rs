use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Option<AccountId>,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountId(pub i32);

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
  pub id: Option<i32>,
  pub exp: DateTime<Utc>,
  pub nbf: DateTime<Utc>,
}
