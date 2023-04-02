use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DinnerOrders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DinnerOrders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DinnerOrders::UserId)
                            .integer()
                            // user can have multiple dinner orders
                            // .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DinnerOrders::CollectionDate)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DinnerOrders::Status)
                            .tiny_unsigned()
                            .not_null(),
                    )
                    /* .col(
                        ColumnDef::new(DinnerOrders::CollectionCode)
                            .integer()
                            .unique_key()
                            .not_null()
                            .auto_increment(),
                    ) */
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DinnerOrders::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum DinnerOrders {
    Table,
    Id,
    UserId,
    CollectionDate,
    Status,
    // CollectionCode,
}
