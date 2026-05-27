# RustBasic JWT Auth

JWT Authentication package for the RustBasic Framework.

- 🔑 **Magic Scaffolding**: Automatically creates `Users` and `JwtBlacklist` migrations and models.
- 🛡️ **Middleware**: Protect your routes with `jwt_auth_middleware` (includes automatic blacklist check).
- 🚫 **Token Invalidation**: Easily invalidate tokens (blacklist) for secure logout.
- 🔄 **Token Refresh**: Refresh old tokens for new ones with automatic invalidation.
- 👤 **Request Scope**: Access the authenticated user's claims in controllers using `get_current_user()`.
- 🛠️ **CLI Tool**: Manual scaffolding and secret generation.

## Installation

Add `rustbasic-jwt` to your `Cargo.toml`:

```toml
[dependencies]
rustbasic-jwt = "0.0.4"
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
use rustbasic_core::{Router, get, from_fn};
use rustbasic_jwt::jwt_auth_middleware;

pub fn router() -> Router {
    Router::new()
        .route("/api/profile", get(profile_handler))
        .layer(from_fn(jwt_auth_middleware))
}
```

### 3. Accessing Authenticated User

In your controller handlers:

```rust
use rustbasic_core::{Response, ResponseHelper};
use rustbasic_jwt::get_current_user;

pub async fn profile_handler() -> Response {
    let claims = match get_current_user() {
        Some(c) => c,
        None => return ResponseHelper::error("Unauthorized"),
    };

    ResponseHelper::json(serde_json::json!({
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
