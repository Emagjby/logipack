use core_data::repository::offices_repo::{self, OfficeError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Error)]
pub enum DeleteOfficeError {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    DeleteOfficeError(#[from] OfficeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn delete_office(
    db: &DatabaseConnection,
    actor: &ActorContext,
    id: Uuid,
) -> Result<Uuid, DeleteOfficeError> {
    // Only admin can delete offices
    if !actor.is_admin() {
        return Err(DeleteOfficeError::Forbidden);
    }

    offices_repo::OfficesRepo::delete_office(db, id)
        .await
        .map_err(|e| match e {
            OfficeError::RecordNotFound => DeleteOfficeError::NotFound,
            other => DeleteOfficeError::DeleteOfficeError(other),
        })?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::OfficeDeleted,
            entity_type: Some(AuditEntityType::Office),
            entity_id: Some(id.to_string()),
            entity_label: Some(format!("Office {}", id)),
            office_id: Some(id),
            office_label: Some(format!("Office {}", id)),
            target_route: Some(format!("/app/admin/offices/{}", id)),
            metadata_json: None,
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(id)
}
