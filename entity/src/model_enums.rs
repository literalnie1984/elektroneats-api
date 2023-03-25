use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
}
