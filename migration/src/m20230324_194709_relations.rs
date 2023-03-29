use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_dinnerOrders_user")
                    .from(DinnerOrders::Table, DinnerOrders::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        //This fails because reasons
        /* manager
        .create_foreign_key(
            sea_query::ForeignKey::create()
                .name("FK_UDO_dinner")
                .from(UserDinnerOrders::Table, UserDinnerOrders::DinnerId)
                .to(Dinner::Table, Dinner::Id)
                .on_delete(ForeignKeyAction::Restrict)
                .on_update(ForeignKeyAction::Restrict)
                .to_owned(),
        )
        .await?; */

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_extrasOrder_UDO")
                    .from(ExtrasOrder::Table, ExtrasOrder::UserDinnerId)
                    .to(UserDinnerOrders::Table, UserDinnerOrders::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_extrasDinner_dinner")
                    .from(ExtrasDinner::Table, ExtrasDinner::DinnerId)
                    .to(Dinner::Table, Dinner::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .table(ExtrasDinner::Table)
                    .name("FK_extrasDinner_dinner")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .table(ExtrasOrder::Table)
                    .name("FK_extrasOrder_UDO")
                    .to_owned(),
            )
            .await?;

        /* manager
        .drop_foreign_key(
            sea_query::ForeignKey::drop()
                .table(UserDinnerOrders::Table)
                .name("FK_UDO_dinner")
                .to_owned(),
        )
        .await?; */
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .table(DinnerOrders::Table)
                    .name("FK_dinnerOrders_user")
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}

#[derive(Iden)]
enum DinnerOrders {
    Table,
    UserId,
}

#[derive(Iden)]
enum ExtrasOrder {
    Table,
    Id,
    UserDinnerId,
    ExtrasId,
}

#[derive(Iden)]
enum ExtrasDinner {
    Table,
    DinnerId,
}

#[derive(Iden)]
enum Dinner {
    Table,
    Id,
}

#[derive(Iden)]
enum UserDinnerOrders {
    Table,
    Id,
    DinnerId,
}
