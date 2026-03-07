use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub occurred_at: DateTimeWithTimeZone,
    pub actor_user_id: Option<Uuid>,
    pub actor_display_name: Option<String>,
    pub action_key: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub entity_label: Option<String>,
    pub office_id: Option<Uuid>,
    pub office_label: Option<String>,
    pub target_route: Option<String>,
    pub metadata_json: Option<JsonValue>,
    pub request_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("audit_events has no relations")
    }
}

impl ActiveModelBehavior for ActiveModel {}
