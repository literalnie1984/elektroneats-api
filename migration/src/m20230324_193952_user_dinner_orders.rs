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
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserDinnerOrders::DinnerId)
                            .integer()
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
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_UDO_U")
                            .from_tbl(UserDinnerOrders::Table)
                            .from_col(UserDinnerOrders::DinnerId)
                            .to_tbl(Dinner::Table)
                            .to_col(Dinner::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("unique_user_dinner_orders")
                    .table(UserDinnerOrders::Table)
                    .col(UserDinnerOrders::OrderId)
                    .col(UserDinnerOrders::DinnerId)
                    .unique()
                    .to_owned()
            )
            .await?;

        // manager
        //     .create_foreign_key(
        //         sea_query::ForeignKey::create()
        //                 .name("FK_UDO_DO")
        //                 .from_tbl(UserDinnerOrders::Table)
        //                 .from_col(UserDinnerOrders::OrderId)
        //                 .to_tbl(DinnerOrders::Table)
        //                 .to_col(DinnerOrders::Id)
        //                 .on_delete(ForeignKeyAction::Restrict)
        //                 .on_update(ForeignKeyAction::Restrict)
        //                 .to_owned()
        //     )
        //     .await?;

        Ok(())
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

#[derive(Iden)]
enum Dinner {
    Table,
    Id,
}
