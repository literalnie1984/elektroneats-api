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
            .create_table(
                Table::create()
                    .table(Dinner::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Dinner::Id)
                            .integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Dinner::Name).string().not_null())
                    .col(ColumnDef::new(Dinner::Price).decimal_len(6, 2).not_null())
                    .col(ColumnDef::new(Dinner::Image).string().not_null())
                    .col(ColumnDef::new(Dinner::WeekDay).tiny_unsigned().not_null())
                    .col(ColumnDef::new(Dinner::MaxSupply).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Extras::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Extras::Id)
                            .integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Extras::Name).string().not_null())
                    .col(ColumnDef::new(Extras::Price).decimal_len(6, 2).not_null())
                    .col(ColumnDef::new(Extras::Image).string().not_null())
                    .col(
                        ColumnDef::new(Extras::Type)
                            .enumeration(Extras::Type, ExtrasType::iter().skip(1)),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Dinner::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Extras::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Dinner {
    Table,
    Id,
    Name,
    Price,
    Image,
    WeekDay,
    MaxSupply,
}

#[derive(Iden)]
enum Extras {
    Table,
    Id,
    Name,
    Price,
    Image,
    Type,
}

#[derive(Iden, EnumIter)]
pub enum ExtrasType {
    Table,
    Filler,
    Beverage,
    Salad,
}

/* #[derive(Iden, EnumIter)]
pub enum WeekDay {
    Table,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
} */
