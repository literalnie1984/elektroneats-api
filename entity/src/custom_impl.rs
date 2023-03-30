use sea_orm::{Related, RelationDef, RelationTrait, Linked};

use crate::{dinner, extras, extras_dinner, extras_order};

impl Related<crate::extras::Entity> for crate::dinner::Entity {
    fn to() -> RelationDef {
        crate::extras_dinner::Relation::Extras.def()
    }

    fn via() -> Option<RelationDef> {
        Some(crate::extras_dinner::Relation::Dinner.def().rev())
    }
}

impl Related<crate::dinner::Entity> for crate::dinner_orders::Entity {
    fn to() -> RelationDef {
        crate::user_dinner_orders::Relation::Dinner.def()
    }

    fn via() -> Option<RelationDef> {
        Some(crate::user_dinner_orders::Relation::DinnerOrders.def().rev())
    }
}


impl Related<crate::extras::Entity> for crate::user_dinner_orders::Entity {
    fn to() -> RelationDef {
        crate::extras_order::Relation::Extras.def()
    }

    fn via() -> Option<RelationDef> {
        Some(crate::extras_order::Relation::UserDinnerOrders.def().rev())
    }
}

#[derive(Debug)]
pub struct DinnerToExtras;

impl Linked for DinnerToExtras {
    type FromEntity = dinner::Entity;

    type ToEntity = extras::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            extras_dinner::Relation::Dinner.def().rev(),
            extras_dinner::Relation::Extras.def(),
        ]
    }
}