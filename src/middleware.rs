use rustbasic_core::requests::Request;
use rustbasic_core::middleware::Next;
use rustbasic_core::router::Response;
use rustbasic_core::responses::ResponseHelper;
use crate::service::JwtService;
use crate::claims::Claims;

rustbasic_core::tokio::task_local! {
    pub static CURRENT_USER: Claims;
}

/// Helper function to retrieve the currently authenticated user's claims in request scope
pub fn get_current_user() -> Option<Claims> {
    CURRENT_USER.try_with(|c| c.clone()).ok()
}

/// Middleware to extract client's Bearer token, validate it, and bind claims to request scope
pub async fn jwt_auth_middleware(
    req: Request,
    next: Next,
) -> Response {
    let auth_header = req.headers
        .get("authorization")
        .filter(|h| h.starts_with("Bearer "));

    let token = match auth_header {
        Some(header) => &header[7..],
        None => {
            return ResponseHelper::error("Token not provided");
        }
    };

    let jwt_service = JwtService::new().with_db(req.state.db.clone());
    match jwt_service.validate_token(token).await {
        Ok(claims) => {
            // Bind claims to task-local context so it is available thread-safely throughout the request lifetime
            CURRENT_USER.scope(claims, next.run(req)).await
        }
        Err(e) => {
            ResponseHelper::error(&format!("Invalid token: {}", e))
        }
    }
}
