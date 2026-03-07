use core_data::repository::employees_repo::{self, EmployeeError};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Error)]
pub enum DeleteEmployeeError {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    EmployeeError(EmployeeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn delete_employee(
    db: &DatabaseConnection,
    actor: &ActorContext,
    id: Uuid,
) -> Result<Uuid, DeleteEmployeeError> {
    // Only admin can delete employees
    if !actor.is_admin() {
        return Err(DeleteEmployeeError::Forbidden);
    }

    employees_repo::EmployeesRepo::delete_employee(db, id)
        .await
        .map_err(|e| match e {
            EmployeeError::RecordNotFound => DeleteEmployeeError::NotFound,
            other => DeleteEmployeeError::EmployeeError(other),
        })?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::EmployeeDeleted,
            entity_type: Some(AuditEntityType::Employee),
            entity_id: Some(id.to_string()),
            entity_label: Some(format!("Employee {}", id)),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/employees/{}", id)),
            metadata_json: None,
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(id)
}
