use core_data::repository::offices_repo::{self, OfficeError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};
use crate::validation::office::{
    OfficeValidationError, validate_address, validate_city, validate_name,
};

#[derive(Debug, Clone)]
pub struct UpdateOffice {
    pub id: Uuid,
    pub name: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Error)]
pub enum UpdateOfficeError {
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(#[from] OfficeValidationError),
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    UpdateOfficeError(#[from] OfficeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn update_office(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: UpdateOffice,
) -> Result<Uuid, UpdateOfficeError> {
    // Only admin can update offices
    if !actor.is_admin() {
        return Err(UpdateOfficeError::Forbidden);
    }

    if let Some(ref name) = input.name {
        validate_name(name)?;
    }

    if let Some(ref city) = input.city {
        validate_city(city)?;
    }

    if let Some(ref address) = input.address {
        validate_address(address)?;
    }

    let name = input.name.clone();
    let city = input.city.clone();
    let address = input.address.clone();

    offices_repo::OfficesRepo::update_office(
        db,
        input.id,
        name.clone(),
        city.clone(),
        address.clone(),
    )
    .await
    .map_err(|e| match e {
        OfficeError::RecordNotFound => UpdateOfficeError::NotFound,
        other => UpdateOfficeError::UpdateOfficeError(other),
    })?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::OfficeUpdated,
            entity_type: Some(AuditEntityType::Office),
            entity_id: Some(input.id.to_string()),
            entity_label: name
                .clone()
                .or_else(|| Some(format!("Office {}", input.id))),
            office_id: Some(input.id),
            office_label: Some(format!("Office {}", input.id)),
            target_route: Some(format!("/app/admin/offices/{}", input.id)),
            metadata_json: Some(serde_json::json!({
                "name": name,
                "city": city,
                "address": address,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(input.id)
}
