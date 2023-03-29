//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.1

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "type")]
pub enum Type {
    #[sea_orm(string_value = "soup")]
    Soup,
    #[sea_orm(string_value = "main")]
    Main,
}
