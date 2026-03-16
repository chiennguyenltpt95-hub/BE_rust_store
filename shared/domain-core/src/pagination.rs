use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u64,
    pub size: u64,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self { page: 0, size: 20 }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
    pub total_pages: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, request: &PageRequest) -> Self {
        let total_pages = (total + request.size - 1) / request.size;
        Self {
            items,
            total,
            page: request.page,
            size: request.size,
            total_pages,
        }
    }
}
