use rustbasic_core::model;

model! {
    table: "jwt_blacklists",
    Model {
        pub id: i32,
        pub jti: String,
        pub exp: i64,
        pub created_at: Option<rustbasic_core::chrono::NaiveDateTime>,
    }
}
