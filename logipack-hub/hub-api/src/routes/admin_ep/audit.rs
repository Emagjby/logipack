use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
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
    page: Option<u64>,
    cursor: Option<String>,
    actor: Option<String>,
    entity_type: Option<String>,
    action: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_audit_handler))
}

fn parse_date_start(raw: Option<&str>) -> Result<Option<DateTime<chrono::FixedOffset>>, ApiError> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let datetime = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ApiError::bad_request("invalid_date", "Date filter is invalid"))?;
        return Ok(Some(
            DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc).fixed_offset(),
        ));
    }

    DateTime::parse_from_rfc3339(value)
        .map(Some)
        .map_err(|_| ApiError::bad_request("invalid_date", "Date filter is invalid"))
}

fn parse_date_end(raw: Option<&str>) -> Result<Option<DateTime<chrono::FixedOffset>>, ApiError> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let next_day = date
            .checked_add_signed(Duration::days(1))
            .ok_or_else(|| ApiError::bad_request("invalid_date", "Date filter is invalid"))?;
        let datetime = next_day
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ApiError::bad_request("invalid_date", "Date filter is invalid"))?;
        return Ok(Some(
            DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc).fixed_offset(),
        ));
    }

    DateTime::parse_from_rfc3339(value)
        .map(Some)
        .map_err(|_| ApiError::bad_request("invalid_date", "Date filter is invalid"))
}

async fn list_audit_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<AuditListQuery>,
) -> Result<Json<ListAuditResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let page_number = query.page.unwrap_or(1).max(1);
    let offset = if query.cursor.is_some() {
        0
    } else {
        (page_number - 1).saturating_mul(limit)
    };
    let cursor = query
        .cursor
        .as_deref()
        .map(core_application::audit::AuditCursor::decode)
        .transpose()
        .map_err(|_| ApiError::bad_request("invalid_cursor", "Cursor is invalid"))?;
    let filters = core_application::audit::AuditListFilters {
        actor: query.actor.clone(),
        entity_type: query.entity_type.clone(),
        action_key: query.action.clone(),
        from: parse_date_start(query.from.as_deref())?,
        to: parse_date_end(query.to.as_deref())?,
    };

    let page = core_application::audit::list_audit_events(
        &state.db,
        &actor,
        limit,
        cursor.as_ref(),
        offset,
        &filters,
    )
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
            total_count: page.total_count,
            next_cursor: page.next_cursor.map(|value| value.encode()),
            has_next: page.has_next,
        },
    }))
}
