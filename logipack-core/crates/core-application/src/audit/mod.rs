pub mod emit;
pub mod types;

use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::actor::ActorContext;

pub use core_data::repository::audit_repo::{AuditCursor, PaginatedAuditEvents};
pub use emit::{AuditError, emit_audit_event};
pub use types::{AuditActionKey, AuditEntityType, AuditEventInput};

#[derive(Debug, Error)]
pub enum ListAuditEventsError {
    #[error("forbidden")]
    Forbidden,
    #[error("audit repository error: {0}")]
    Repo(#[from] core_data::repository::audit_repo::AuditRepoError),
}

pub async fn list_audit_events(
    db: &DatabaseConnection,
    actor: &ActorContext,
    limit: u64,
    cursor: Option<&AuditCursor>,
) -> Result<PaginatedAuditEvents, ListAuditEventsError> {
    if !actor.is_admin() {
        return Err(ListAuditEventsError::Forbidden);
    }

    core_data::repository::audit_repo::AuditRepo::list_paginated(db, limit, cursor)
        .await
        .map_err(Into::into)
}
