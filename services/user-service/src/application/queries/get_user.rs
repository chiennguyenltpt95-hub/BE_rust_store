use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct GetUserQuery {
    pub user_id: Uuid,
}

/// View model (DTO) trả về cho client
#[derive(Debug, Serialize, Deserialize)]
pub struct UserView {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
