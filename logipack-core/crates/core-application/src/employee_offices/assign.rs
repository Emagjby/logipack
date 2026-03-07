use core_data::repository::employee_offices_repo::{EmployeeOfficeError, EmployeeOfficesRepo};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Clone)]
pub struct AssignOffice {
    pub employee_id: Uuid,
    pub office_id: Uuid,
}

#[derive(Debug, Error)]
pub enum AssignOfficeError {
    #[error("forbidden")]
    Forbidden,
    #[error("employee not found")]
    EmployeeNotFound,
    #[error("office not found")]
    OfficeNotFound,
    #[error("employee already assigned to office")]
    AlreadyAssigned,
    #[error("{0}")]
    AssignError(#[from] EmployeeOfficeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn assign_office(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: AssignOffice,
) -> Result<(), AssignOfficeError> {
    if !actor.is_admin() {
        return Err(AssignOfficeError::Forbidden);
    }

    let inserted = EmployeeOfficesRepo::assign_office(db, input.employee_id, input.office_id)
        .await
        .map_err(|e| match e {
            EmployeeOfficeError::EmployeeNotFound => AssignOfficeError::EmployeeNotFound,
            EmployeeOfficeError::OfficeNotFound => AssignOfficeError::OfficeNotFound,
            other => AssignOfficeError::AssignError(other),
        })?;

    if !inserted {
        return Err(AssignOfficeError::AlreadyAssigned);
    }

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::EmployeeAssignedToOffice,
            entity_type: Some(AuditEntityType::Employee),
            entity_id: Some(input.employee_id.to_string()),
            entity_label: Some(format!("Employee {}", input.employee_id)),
            office_id: Some(input.office_id),
            office_label: Some(format!("Office {}", input.office_id)),
            target_route: Some(format!(
                "/app/admin/employees/{}/offices",
                input.employee_id
            )),
            metadata_json: None,
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(())
}
