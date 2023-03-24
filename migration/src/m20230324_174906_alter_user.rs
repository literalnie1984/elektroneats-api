use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = sea_query::Table::alter()
            .table(User::Table)
            .add_column(
                ColumnDef::new(Alias::new("admin"))
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .to_owned();
        manager.alter_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = sea_query::Table::alter()
            .table(User::Table)
            .drop_column(Alias::new("admin"))
            .to_owned();
        manager.alter_table(table).await
    }
}

#[derive(Iden)]
enum User {
    Table,
}
