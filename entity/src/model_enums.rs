use sea_orm::{DeriveActiveEnum, EnumIter, strum::FromRepr};
use serde::{Deserialize, Serialize};

#[derive(DeriveActiveEnum, EnumIter, Serialize, Deserialize, Clone)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[repr(u8)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
}

#[derive(DeriveActiveEnum, EnumIter, Serialize, Deserialize, Clone, FromRepr)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[repr(u8)]
pub enum Status {
    Paid = 0,
    Prepared = 1,
    Ready = 2,
    Collected = 3,
}