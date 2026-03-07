use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use core_application::actor::ActorContext;
use serde::Deserialize;

use crate::{
    dto::audit::{AuditEventDto, AuditPageDto, ListAuditResponse},
    error::ApiError,
    policy,
    state::AppState,
};

const DEFAULT_LIMIT: u64 = 10;
const MAX_LIMIT: u64 = 100;

#[derive(Debug, Deserialize)]
struct AuditListQuery {
    limit: Option<u64>,
    cursor: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_audit_handler))
}

async fn list_audit_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<AuditListQuery>,
) -> Result<Json<ListAuditResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let cursor = query
        .cursor
        .as_deref()
        .map(core_application::audit::AuditCursor::decode)
        .transpose()
        .map_err(|_| ApiError::bad_request("invalid_cursor", "Cursor is invalid"))?;

    let page =
        core_application::audit::list_audit_events(&state.db, &actor, limit, cursor.as_ref())
            .await
            .map_err(|e| match e {
                core_application::audit::ListAuditEventsError::Forbidden => {
                    ApiError::forbidden("access_denied", "Access denied")
                }
                core_application::audit::ListAuditEventsError::Repo(err) => {
                    ApiError::internal(err.to_string())
                }
            })?;

    let events = page
        .rows
        .into_iter()
        .map(|row| AuditEventDto {
            id: row.id.to_string(),
            occurred_at: row.occurred_at.to_rfc3339(),
            actor_user_id: row.actor_user_id.map(|value| value.to_string()),
            actor_display_name: row.actor_display_name,
            action_key: row.action_key,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            entity_label: row.entity_label,
            office_id: row.office_id.map(|value| value.to_string()),
            office_label: row.office_label,
            target_route: row.target_route,
            metadata: row.metadata_json,
        })
        .collect();

    Ok(Json(ListAuditResponse {
        events,
        page: AuditPageDto {
            limit,
            next_cursor: page.next_cursor.map(|value| value.encode()),
            has_next: page.has_next,
        },
    }))
}
