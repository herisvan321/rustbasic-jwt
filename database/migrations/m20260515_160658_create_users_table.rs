use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum Users {
    Table, Id, Name, Email, Password, CreatedAt, UpdatedAt,
}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260515_160658_create_users_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}
