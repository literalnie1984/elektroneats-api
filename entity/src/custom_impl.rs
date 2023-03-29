use sea_orm::{Related, RelationDef, RelationTrait};

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

// impl Related<crate::extras::Entity> for crate::user_dinner_orders::Entity {
//     fn to() -> RelationDef {
//         crate::extras_order::Relation::Extras.def()
//     }

//     fn via() -> Option<RelationDef> {
//         Some(crate::extras_order::Relation::UserDinnerOrders.def().rev())
//     }
// }