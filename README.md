# RustBasic JWT Auth

JWT Authentication package for the RustBasic Framework.

## Features

- 🔑 **Magic Scaffolding**: Automatically creates `Users` and `JwtBlacklist` migrations and models.
- 🛡️ **Middleware**: Protect your Axum routes with `jwt_auth_middleware` (includes automatic blacklist check).
- 🚫 **Token Invalidation**: Easily invalidate tokens (blacklist) for secure logout.
- 🔄 **Token Refresh**: Refresh old tokens for new ones with automatic invalidation.
- 👤 **Easy Extractor**: Access the authenticated user in controllers using `AuthUser`.
- 🛠️ **CLI Tool**: Manual scaffolding and secret generation.

## Installation

Add `rustbasic-jwt` to your `Cargo.toml`:

```toml
[dependencies]
rustbasic-jwt = "0.0.2"
```

## Configuration

Settings are managed via your `.env` file:

```bash
# --- JWT CONFIG ---
JWT_SECRET=your-random-secret-key
JWT_TTL=60              # Token lifetime in minutes
JWT_REFRESH_TTL=20160   # Refresh token lifetime
JWT_ALGO=HS256          # Algorithm (HS256, HS384, HS512)
```

## Usage

### 1. Scaffolding

The package automatically scaffolds the necessary files on build. You can also run it manually:

```bash
cargo run --bin rustbasic-jwt install
```

This will create:
- `JWT_SECRET` etc in `.env`.
- `create_users_table` & `create_jwt_blacklists_table` migrations.
- `User` & `JwtBlacklist` models.

### 2. Protecting Routes

In your route definitions:

```rust
use rustbasic_jwt::{jwt_auth_middleware, HasDatabase};
use axum::{routing::get, Router, middleware};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/profile", get(profile_handler))
        .layer(middleware::from_fn_with_state(state, jwt_auth_middleware::<AppState>))
}
```

### 3. Accessing Authenticated User

In your controller handlers:

```rust
use rustbasic_jwt::AuthUser;

pub async fn profile_handler(AuthUser(claims): AuthUser) -> impl IntoResponse {
    Json(json!({
        "message": "Hello user!",
        "user_id": claims.sub,
        "data": claims.user_data
    }))
}
```

### 4. Generating & Refreshing Tokens

```rust
use rustbasic_jwt::JwtService;

// With database support (required for blacklist/refresh)
let jwt = JwtService::new().with_db(db);

// Generate
let token = jwt.generate_token(user.id.to_string(), json!({"email": user.email}))?;

// Refresh (invalidates old token and returns new one)
let new_token = jwt.refresh_token(old_token).await?;

// Logout
jwt.invalidate_token(token).await?;
```


## Automation

The package automatically scaffolds the necessary files (Migrations, Models, and .env settings) during the first `cargo build`. No manual installation commands are required.
