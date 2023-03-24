use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Shop::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Shop::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Shop::Name).string().not_null())
                    .col(ColumnDef::new(Shop::Price).decimal_len(6, 2).not_null())
                    .col(ColumnDef::new(Shop::Supply).integer().unsigned().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ShopOrders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ShopOrders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ShopOrders::UserId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ShopOrders::OrderId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ShopOrders::Completed).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShopOrders::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Shop::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Shop {
    Table,
    Id,
    Name,
    //Category - add in a later migration (it will be an enum)
    Price,
    Supply,
}

#[derive(Iden)]
enum ShopOrders {
    Table,
    Id,
    UserId,
    OrderId,
    Completed,
}
