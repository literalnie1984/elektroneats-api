use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExtrasOrder::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExtrasOrder::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExtrasOrder::UserDinnerId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtrasOrder::ExtrasId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ExtrasDinner::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExtrasDinner::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExtrasDinner::DinnerId).integer().not_null())
                    .col(ColumnDef::new(ExtrasDinner::ExtrasId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("UNIQ_extras_dinner")
                    .table(ExtrasDinner::Table)
                    .col(ExtrasDinner::DinnerId)
                    .col(ExtrasDinner::ExtrasId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_extras_order")
                    .from_tbl(ExtrasOrder::Table)
                    .from_col(ExtrasOrder::ExtrasId)
                    .to_tbl(Extras::Table)
                    .to_col(Extras::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_extras_dinner")
                    .from_tbl(ExtrasDinner::Table)
                    .from_col(ExtrasDinner::ExtrasId)
                    .to_tbl(Extras::Table)
                    .to_col(Extras::Id)
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
                    .table(ExtrasOrder::Table)
                    .name("FK_extras_order")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .table(ExtrasDinner::Table)
                    .name("FK_extras_dinner")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ExtrasOrder::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ExtrasDinner::Table).to_owned())
            .await?;
        Ok(())
    }
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
    Id,
    DinnerId,
    ExtrasId,
}

#[derive(Iden)]
enum Extras {
    Table,
    Id,
}
