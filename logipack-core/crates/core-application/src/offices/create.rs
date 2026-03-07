use core_data::repository::offices_repo::{self, OfficeError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};
use crate::validation::office::{OfficeValidationError, validate_office};

#[derive(Debug, Clone)]
pub struct CreateOffice {
    pub name: String,
    pub city: String,
    pub address: String,
}

#[derive(Debug, Error)]
pub enum CreateOfficeError {
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(#[from] OfficeValidationError),
    #[error("{0}")]
    OfficeCreationError(#[from] OfficeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn create_office(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: CreateOffice,
) -> Result<Uuid, CreateOfficeError> {
    // Only admin can create offices
    if !actor.is_admin() {
        return Err(CreateOfficeError::Forbidden);
    }

    validate_office(&input.name, &input.city, &input.address)?;

    let office_id = Uuid::new_v4();

    let CreateOffice {
        name,
        city,
        address,
    } = input;

    offices_repo::OfficesRepo::create_office(
        db,
        office_id,
        name.clone(),
        city.clone(),
        address.clone(),
    )
    .await?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::OfficeCreated,
            entity_type: Some(AuditEntityType::Office),
            entity_id: Some(office_id.to_string()),
            entity_label: Some(name),
            office_id: Some(office_id),
            office_label: Some(format!("Office {}", office_id)),
            target_route: Some(format!("/app/admin/offices/{}", office_id)),
            metadata_json: Some(serde_json::json!({
                "city": city,
                "address": address,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(office_id)
}
