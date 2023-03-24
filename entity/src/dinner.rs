//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.1

use super::sea_orm_active_enums::Type;
use super::sea_orm_active_enums::WeekDay;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dinner")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "Decimal(Some((6, 2)))")]
    pub price: Decimal,
    pub image: String,
    pub week_day: WeekDay,
    pub max_supply: i32,
    pub r#type: Type,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::extras_dinner::Entity")]
    ExtrasDinner,
}

impl Related<super::extras_dinner::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExtrasDinner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}