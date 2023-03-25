use sea_orm::{Related, RelationDef, RelationTrait};

impl Related<crate::extras::Entity> for crate::dinner::Entity {
    fn to() -> RelationDef {
        crate::extras_dinner::Relation::Extras.def()
    }

    fn via() -> Option<RelationDef> {
        Some(crate::extras_dinner::Relation::Dinner.def().rev())
    }
}