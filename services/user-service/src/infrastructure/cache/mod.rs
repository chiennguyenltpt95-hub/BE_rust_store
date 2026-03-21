use async_trait::async_trait;
use domain_core::error::DomainError;
use redis::{Client, Commands};

/// Trait cho cache — application layer chỉ biết trait này
#[async_trait]
pub trait CacheService: Send + Sync {
    fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), DomainError>;
    fn get(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn delete(&self, key: &str) -> Result<(), DomainError>;
}

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, DomainError> {
        let client = Client::open(redis_url).map_err(|e| {
            DomainError::InfrastructureError(format!("Redis connection error: {}", e))
        })?;
        Ok(RedisCache { client })
    }
}

#[async_trait]
impl CacheService for RedisCache {
    fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), DomainError> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| DomainError::InfrastructureError(format!("Redis error: {}", e)))?;
        let _: () = conn.set_ex(key, value, ttl_seconds)
            .map_err(|e| DomainError::InfrastructureError(format!("Redis set error: {}", e)))?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<String>, DomainError> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| DomainError::InfrastructureError(format!("Redis error: {}", e)))?;
        let result: Option<String> = conn
            .get(key)
            .map_err(|e| DomainError::InfrastructureError(format!("Redis get error: {}", e)))?;
        Ok(result)
    }

    fn delete(&self, key: &str) -> Result<(), DomainError> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| DomainError::InfrastructureError(format!("Redis error: {}", e)))?;
        let _: () = conn.del(key)
            .map_err(|e| DomainError::InfrastructureError(format!("Redis del error: {}", e)))?;
        Ok(())
    }
}
