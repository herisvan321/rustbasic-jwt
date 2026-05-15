use std::fs;
use std::path::PathBuf;
use std::env;

fn main() {
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let project_root = match env::var("PWD") {
        Ok(pwd) => PathBuf::from(pwd),
        Err(_) => match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return,
        },
    };

    if !project_root.join("Cargo.toml").exists() {
        return;
    }

    // Don't scaffold if we are developing the package itself
    if project_root.join("src/lib.rs").exists() && !project_root.join("Cargo.toml").to_str().unwrap_or("").contains("cobadata") {
        // This is a simple heuristic to detect if we are in the library project itself
        if fs::read_to_string(project_root.join("Cargo.toml")).unwrap_or_default().contains("name = \"rustbasic-jwt\"") {
             return;
        }
    }

    // 1. Generate JWT_SECRET in .env if not exists
    let env_path = project_root.join(".env");
    if env_path.exists() {
        let content = fs::read_to_string(&env_path).unwrap_or_default();
        if !content.contains("JWT_SECRET") {
            let mut file = fs::OpenOptions::new().append(true).open(&env_path).unwrap();
            use std::io::Write;
            let secret = uuid::Uuid::new_v4().to_string();
            writeln!(file, "\n# --- JWT CONFIG ---").ok();
            writeln!(file, "JWT_SECRET={}", secret).ok();
            writeln!(file, "JWT_TTL=60").ok();
            writeln!(file, "JWT_REFRESH_TTL=20160").ok();
            writeln!(file, "JWT_ALGO=HS256").ok();
            println!("cargo:warning=🔑 rustbasic-jwt: Konfigurasi JWT baru ditambahkan ke .env");
        }
    } else {
        let secret = uuid::Uuid::new_v4().to_string();
        let content = format!("JWT_SECRET={}\nJWT_TTL=60\nJWT_REFRESH_TTL=20160\nJWT_ALGO=HS256\n", secret);
        fs::write(&env_path, content).ok();
        println!("cargo:warning=🔑 rustbasic-jwt: File .env baru dibuat dengan konfigurasi JWT");
    }

    // 2. Create Migration for Users (if needed)
    let migrations_dir = project_root.join("database/migrations");
    fs::create_dir_all(&migrations_dir).ok();

    let existing_migrations = fs::read_dir(&migrations_dir)
        .map(|dir| dir.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("create_users_table")))
        .unwrap_or(false);

    if !existing_migrations {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let migration_name = format!("m{}_create_users_table", timestamp);
        let migration_path = migrations_dir.join(format!("{}.rs", migration_name));

        let migration_template = format!(
r#"use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum Users {{
    Table, Id, Name, Email, Password, CreatedAt, UpdatedAt,
}}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}
}}

#[async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.create_table(
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(ColumnDef::new(Users::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Users::Name).string().not_null())
                .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                .col(ColumnDef::new(Users::Password).string().not_null())
                .col(ColumnDef::new(Users::CreatedAt).date_time().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Users::UpdatedAt).date_time().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }}
}}
"#, migration_name = migration_name);

        if fs::write(&migration_path, migration_template).is_ok() {
            update_migration_mod_rs(&project_root, &migration_name);
        }
    }

    // 2.5 Create Blacklist Migration
    let existing_blacklist = fs::read_dir(&migrations_dir)
        .map(|dir| dir.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("create_jwt_blacklists_table")))
        .unwrap_or(false);

    if !existing_blacklist {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let migration_name = format!("m{}_create_jwt_blacklists_table", timestamp);
        let migration_path = migrations_dir.join(format!("{}.rs", migration_name));

        let migration_template = format!(
r#"use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum JwtBlacklists {{
    Table, Id, Jti, Exp, CreatedAt,
}}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}
}}

#[async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.create_table(
            Table::create()
                .table(JwtBlacklists::Table)
                .if_not_exists()
                .col(ColumnDef::new(JwtBlacklists::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(JwtBlacklists::Jti).string().not_null().unique_key())
                .col(ColumnDef::new(JwtBlacklists::Exp).big_integer().not_null())
                .col(ColumnDef::new(JwtBlacklists::CreatedAt).date_time().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.drop_table(Table::drop().table(JwtBlacklists::Table).to_owned()).await?;
        Ok(())
    }}
}}
"#, migration_name = migration_name);

        if fs::write(&migration_path, migration_template).is_ok() {
            update_migration_mod_rs(&project_root, &migration_name);
        }
    }

    // 3. Create Model
    let models_dir = project_root.join("src/app/models");
    fs::create_dir_all(&models_dir).ok();

    let model_name = "user";
    let table_name = "users";
    let file_path = models_dir.join(format!("{}.rs", model_name));
    
    if !file_path.exists() {
        let model_template = format!(
r#"use rustbasic_core::sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table_name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#, table_name = table_name);

        if fs::write(&file_path, model_template).is_ok() {
            update_model_mod_rs(&project_root, "User", model_name);
        }
    }

    // 4. Create Blacklist Model
    let model_name = "jwt_blacklist";
    let table_name = "jwt_blacklists";
    let file_path = models_dir.join(format!("{}.rs", model_name));
    
    if !file_path.exists() {
        let model_template = format!(
r#"use rustbasic_core::sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table_name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub jti: String,
    pub exp: i64,
    pub created_at: Option<DateTime>,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#, table_name = table_name);

        if fs::write(&file_path, model_template).is_ok() {
            update_model_mod_rs(&project_root, "JwtBlacklist", model_name);
        }
    }
}

fn update_migration_mod_rs(project_root: &std::path::Path, mod_name: &str) {
    let mod_path = project_root.join("database/migrations/mod.rs");
    if !mod_path.exists() { return; }

    let mut content = fs::read_to_string(&mod_path).unwrap_or_default();

    if !content.contains(&format!("pub mod {};", mod_name)) {
        content.push_str(&format!("\npub mod {};\n", mod_name));
    }

    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern) {
        if let Some(insert_pos) = content[pos..].find("        ]") {
            content.insert_str(pos + insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
        }
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(project_root: &std::path::Path, class_name: &str, snake_name: &str) {
    let mod_path = project_root.join("src/app/models/mod.rs");
    if !mod_path.exists() { return; }

    let content = fs::read_to_string(&mod_path).unwrap_or_default();
    
    // Check for singular and plural versions
    let singular = format!("pub mod {};", snake_name);
    let plural = format!("pub mod {}s;", snake_name);
    
    if content.contains(&singular) || content.contains(&plural) {
        return;
    }

    let mut file = fs::OpenOptions::new().append(true).open(mod_path).unwrap();
    use std::io::Write;
    writeln!(file, "pub mod {};", snake_name).ok();
    writeln!(file, "pub use {}::Entity as {};", snake_name, class_name).ok();
}
