use core_data::repository::clients_repo::{self, ClientError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};
use crate::validation::client::{
    ClientValidationError, validate_email, validate_name, validate_phone,
};

#[derive(Debug, Clone)]
pub struct UpdateClient {
    pub id: Uuid,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Error)]
pub enum UpdateClientError {
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(#[from] ClientValidationError),
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    UpdateClientError(#[from] ClientError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn update_client(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: UpdateClient,
) -> Result<Uuid, UpdateClientError> {
    // Only admin can update clients
    if !actor.is_admin() {
        return Err(UpdateClientError::Forbidden);
    }

    if let Some(ref name) = input.name {
        validate_name(name)?;
    }

    if let Some(ref email) = input.email {
        validate_email(email)?;
    }

    validate_phone(input.phone.as_deref())?;

    let name = input.name.clone();
    let phone = input.phone.clone();
    let email = input.email.clone();

    clients_repo::ClientsRepo::update_client(
        db,
        input.id,
        name.clone(),
        phone.clone(),
        email.clone(),
    )
    .await
    .map_err(|e| match e {
        ClientError::RecordNotFound => UpdateClientError::NotFound,
        other => UpdateClientError::UpdateClientError(other),
    })?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::ClientUpdated,
            entity_type: Some(AuditEntityType::Client),
            entity_id: Some(input.id.to_string()),
            entity_label: name
                .clone()
                .or_else(|| Some(format!("Client {}", input.id))),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/clients/{}", input.id)),
            metadata_json: Some(serde_json::json!({
                "name": name,
                "phone": phone,
                "email": email,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(input.id)
}
