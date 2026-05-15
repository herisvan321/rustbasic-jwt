use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use crate::claims::Claims;
use std::env;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use crate::entities::jwt_blacklist;

pub struct JwtService {
    secret: String,
    db: Option<DatabaseConnection>,
}

impl JwtService {
    pub fn new() -> Self {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret_rustbasic_key_1234567890".to_string());
        Self { secret, db: None }
    }

    pub fn with_db(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    pub fn generate_token(&self, user_id: String, user_data: serde_json::Value) -> Result<String, jsonwebtoken::errors::Error> {
        let ttl = env::var("JWT_TTL").unwrap_or_else(|_| "60".to_string()).parse::<i64>().unwrap_or(60);
        let claims = Claims::new(user_id, user_data, ttl);
        
        let algo = match env::var("JWT_ALGO").unwrap_or_default().as_str() {
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256,
        };

        encode(
            &Header::new(algo),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let algo = match env::var("JWT_ALGO").unwrap_or_default().as_str() {
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256,
        };

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(algo),
        ).map_err(|e| e.to_string())?;

        // Check if blacklisted
        if let Some(db) = &self.db {
            let is_blacklisted = jwt_blacklist::Entity::find()
                .filter(jwt_blacklist::Column::Jti.eq(&token_data.claims.jti))
                .one(db)
                .await
                .map_err(|e| e.to_string())?;

            if is_blacklisted.is_some() {
                return Err("Token is blacklisted".to_string());
            }
        }

        Ok(token_data.claims)
    }

    pub async fn invalidate_token(&self, token: &str) -> Result<(), String> {
        let claims = self.validate_token(token).await?;
        
        if let Some(db) = &self.db {
            let active_model = jwt_blacklist::ActiveModel {
                jti: Set(claims.jti),
                exp: Set(claims.exp),
                ..Default::default()
            };
            active_model.insert(db).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Database connection required for invalidation".to_string())
        }
    }

    pub async fn refresh_token(&self, token: &str) -> Result<String, String> {
        let claims = self.validate_token(token).await?;
        
        // Invalidate old token
        let _ = self.invalidate_token(token).await;
        
        // Generate new token
        self.generate_token(claims.sub, claims.user_data).map_err(|e| e.to_string())
    }
}

impl Default for JwtService {
    fn default() -> Self {
        Self::new()
    }
}
