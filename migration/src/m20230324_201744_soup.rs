use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(Dinner::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("type"))
                            .enumeration(Dinner::Type, Type::iter().skip(1))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(Dinner::Table)
                    .drop_column(Dinner::Type)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Dinner {
    Table,
    Type,
}

#[derive(Iden, EnumIter)]
pub enum Type {
    Table,
    Soup,
    Main,
}
