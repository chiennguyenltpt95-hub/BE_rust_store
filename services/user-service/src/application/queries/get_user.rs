use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug)]
pub struct GetUserQuery {
    pub user_id: Uuid,
}

/// View model (DTO) trả về cho client
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserView {
    #[schema(example = "00000000-0000-0000-0000-000000000000")]
    pub id: Uuid,
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "Nguyen Van A")]
    pub full_name: String,
    #[schema(example = "Customer")]
    pub role: String,
    #[schema(example = "Active")]
    pub status: String,
    pub address: Option<String>,
    pub age: Option<i16>,
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
}
