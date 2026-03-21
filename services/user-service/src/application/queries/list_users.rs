use domain_core::pagination::PageRequest;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUsersQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub role: Option<String>,
    pub status: Option<String>,
}

impl ListUsersQuery {
    pub fn page_request(&self) -> PageRequest {
        PageRequest {
            page: self.page.unwrap_or(0),
            size: self.size.unwrap_or(20).min(100),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserSummary {
    #[schema(example = "00000000-0000-0000-0000-000000000000")]
    pub id: uuid::Uuid,
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
    pub verified: bool,
}
