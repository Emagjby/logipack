use core_data::repository::clients_repo::{self, ClientError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Error)]
pub enum DeleteClientError {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    DeleteClientError(#[from] ClientError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn delete_client(
    db: &DatabaseConnection,
    actor: &ActorContext,
    id: Uuid,
) -> Result<Uuid, DeleteClientError> {
    // Only admin can delete clients
    if !actor.is_admin() {
        return Err(DeleteClientError::Forbidden);
    }

    clients_repo::ClientsRepo::delete_client(db, id)
        .await
        .map_err(|e| match e {
            ClientError::RecordNotFound => DeleteClientError::NotFound,
            other => DeleteClientError::DeleteClientError(other),
        })?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::ClientDeleted,
            entity_type: Some(AuditEntityType::Client),
            entity_id: Some(id.to_string()),
            entity_label: Some(format!("Client {}", id)),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/clients/{}", id)),
            metadata_json: None,
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(id)
}
