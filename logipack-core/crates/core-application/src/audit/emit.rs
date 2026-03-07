use chrono::Utc;
use core_data::repository::audit_repo::{AuditRepo, AuditRepoError, InsertAuditEvent};
use core_data::repository::users_repo::{UserError, UserRepo};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;

use super::types::AuditEventInput;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("audit repository error: {0}")]
    Repo(#[from] AuditRepoError),
    #[error("actor lookup error: {0}")]
    ActorLookup(#[from] UserError),
}

pub async fn emit_audit_event(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: AuditEventInput,
) -> Result<Uuid, AuditError> {
    let actor_display_name = resolve_actor_display_name(db, actor).await?;

    let row = AuditRepo::insert_event(
        db,
        InsertAuditEvent {
            id: Uuid::new_v4(),
            occurred_at: input
                .occurred_at
                .unwrap_or_else(|| Utc::now().fixed_offset()),
            actor_user_id: Some(actor.user_id),
            actor_display_name,
            action_key: input.action_key.as_str().to_string(),
            entity_type: input.entity_type.map(|value| value.as_str().to_string()),
            entity_id: input.entity_id,
            entity_label: input.entity_label,
            office_id: input.office_id,
            office_label: input.office_label,
            target_route: input.target_route,
            metadata_json: input.metadata_json,
            request_id: input.request_id,
        },
    )
    .await?;

    Ok(row.id)
}

async fn resolve_actor_display_name(
    db: &DatabaseConnection,
    actor: &ActorContext,
) -> Result<Option<String>, AuditError> {
    let user = UserRepo::get_by_id(db, actor.user_id).await?;
    let Some(user) = user else {
        return Ok(None);
    };

    if !user.name.trim().is_empty() {
        return Ok(Some(user.name));
    }

    Ok(user.email)
}
