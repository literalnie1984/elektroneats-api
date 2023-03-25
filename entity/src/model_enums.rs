use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}
