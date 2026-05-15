use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
    Json,
};
use crate::service::JwtService;
use serde_json::json;
use axum::extract::State;
use sea_orm::DatabaseConnection;
use rustbasic_core::server::AppState;
use crate::claims::Claims;

pub trait HasDatabase {
    fn db(&self) -> DatabaseConnection;
}

impl HasDatabase for AppState {
    fn db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}

pub async fn jwt_auth_middleware<S>(
    state: State<S>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> 
where
    S: HasDatabase + Clone + Send + Sync + 'static
{
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "));

    let token = match auth_header {
        Some(header) => &header[7..],
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": "Token not provided"
                })),
            ));
        }
    };

    let jwt_service = JwtService::new().with_db(state.db());
    match jwt_service.validate_token(token).await {
        Ok(claims) => {
            // Insert claims into request extensions so controllers can access it
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            let message = format!("Invalid token: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": message
                })),
            ))
        }
    }
}

use axum::{
    extract::FromRequestParts,
};
use http::request::Parts;

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            Ok(AuthUser(claims.clone()))
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": "Unauthorized"
                })),
            ))
        }
    }
}
