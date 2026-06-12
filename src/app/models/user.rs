use rustbasic_core::model;

model! {
    table: "users",
    Model {
        pub id: i32,
        pub name: String,
        pub email: String,
        #[serde(skip_serializing)]
        pub password: String,
        pub created_at: Option<rustbasic_core::chrono::NaiveDateTime>,
        pub updated_at: Option<rustbasic_core::chrono::NaiveDateTime>,
    }
}
