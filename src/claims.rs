use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub iss: String,      // Issuer
    pub sub: String,      // Subject (user id)
    pub iat: i64,         // Issued at
    pub exp: i64,         // Expiration time
    pub nbf: i64,         // Not before
    pub jti: String,      // JWT ID
    pub user_data: serde_json::Value, // Custom data like name, roles, etc.
}

impl Claims {
    pub fn new(sub: String, user_data: serde_json::Value, expires_in_minutes: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::minutes(expires_in_minutes);
        
        Self {
            iss: "rustbasic".to_string(),
            sub,
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            user_data,
        }
    }
}
