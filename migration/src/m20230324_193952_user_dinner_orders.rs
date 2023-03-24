use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserDinnerOrders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserDinnerOrders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserDinnerOrders::OrderId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserDinnerOrders::DinnerId)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_UDO_DO")
                            .from_tbl(UserDinnerOrders::Table)
                            .from_col(UserDinnerOrders::OrderId)
                            .to_tbl(DinnerOrders::Table)
                            .to_col(DinnerOrders::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserDinnerOrders::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserDinnerOrders {
    Table,
    Id,
    OrderId,
    DinnerId,
}

#[derive(Iden)]
enum DinnerOrders {
    Table,
    Id,
}
