use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEventDto {
    pub id: String,
    pub occurred_at: String,
    pub actor_user_id: Option<String>,
    pub actor_display_name: Option<String>,
    pub action_key: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub entity_label: Option<String>,
    pub office_id: Option<String>,
    pub office_label: Option<String>,
    pub target_route: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditPageDto {
    pub limit: u64,
    pub next_cursor: Option<String>,
    pub has_next: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAuditResponse {
    pub events: Vec<AuditEventDto>,
    pub page: AuditPageDto,
}
