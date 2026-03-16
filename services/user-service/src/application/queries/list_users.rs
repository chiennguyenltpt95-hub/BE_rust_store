use serde::{Deserialize, Serialize};
use domain_core::pagination::PageRequest;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub status: String,
}
