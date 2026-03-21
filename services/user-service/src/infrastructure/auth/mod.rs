use chrono::Utc;
use domain_core::error::DomainError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Access token TTL: 15 phút
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;
/// Refresh token TTL: 7 ngày
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 7;

/// Payload bên trong JWT access token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub payload: serde_json::Value,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Tạo access token ngắn hạn (15 phút)
    pub fn generate_access_token<T: Serialize>(
        &self,
        user_id: Uuid,
        payload: T,
    ) -> Result<String, DomainError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            payload: serde_json::to_value(payload).unwrap_or_default(),
            iat: now,
            exp: now + ACCESS_TOKEN_TTL_SECS,
        };
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))
    }

    /// Verify và decode access token
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError> {
        decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|e| DomainError::Unauthorized(format!("Invalid token: {}", e)))
    }
}

/// Tạo refresh token ngẫu nhiên (opaque string, 256-bit entropy)
pub fn generate_refresh_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Hash refresh token bằng SHA-256 trước khi lưu DB
/// → Nếu DB bị leak, attacker không dùng được raw token
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
