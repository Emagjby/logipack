use core_data::repository::clients_repo::{self, ClientError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};
use crate::validation::client::{ClientValidationError, validate_client};

#[derive(Debug, Clone)]
pub struct CreateClient {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Error)]
pub enum CreateClientError {
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(#[from] ClientValidationError),
    #[error("{0}")]
    ClientCreationError(#[from] ClientError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn create_client(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: CreateClient,
) -> Result<Uuid, CreateClientError> {
    // Only admin can create clients
    if !actor.is_admin() {
        return Err(CreateClientError::Forbidden);
    }

    let email = input.email.as_deref().ok_or(CreateClientError::Validation(
        ClientValidationError::InvalidEmail,
    ))?;

    validate_client(&input.name, email, input.phone.as_deref())?;

    let client_id = Uuid::new_v4();

    let CreateClient { name, phone, email } = input;

    clients_repo::ClientsRepo::create_client(
        db,
        client_id,
        name.clone(),
        phone.clone(),
        email.clone(),
    )
    .await?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::ClientCreated,
            entity_type: Some(AuditEntityType::Client),
            entity_id: Some(client_id.to_string()),
            entity_label: Some(name),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/clients/{}", client_id)),
            metadata_json: Some(serde_json::json!({
                "phone": phone,
                "email": email,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(client_id)
}
