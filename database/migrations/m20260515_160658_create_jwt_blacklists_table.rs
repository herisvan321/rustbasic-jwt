use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum JwtBlacklists {
    Table, Id, Jti, Exp, CreatedAt,
}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260515_160658_create_jwt_blacklists_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(JwtBlacklists::Table).to_owned()).await?;
        Ok(())
    }
}
