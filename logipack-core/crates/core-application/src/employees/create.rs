use core_data::repository::employees_repo::{self, EmployeeError};
use core_data::repository::users_repo::{UserError, UserRepo};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Clone)]
pub struct CreateEmployee {
    pub email: String,
}

#[derive(Debug, Error)]
pub enum CreateEmployeeError {
    #[error("forbidden")]
    Forbidden,
    #[error("user not found")]
    UserNotFound,
    #[error("{0}")]
    UserError(UserError),
    #[error("{0}")]
    EmployeeCreationError(#[from] EmployeeError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn create_employee(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: CreateEmployee,
) -> Result<Uuid, CreateEmployeeError> {
    // Only admin can create employees
    if !actor.is_admin() {
        return Err(CreateEmployeeError::Forbidden);
    }

    let user = UserRepo::get_by_email(db, &input.email)
        .await
        .map_err(CreateEmployeeError::UserError)?
        .ok_or(CreateEmployeeError::UserNotFound)?;

    let employee_id = Uuid::new_v4();
    let created_id =
        employees_repo::EmployeesRepo::create_employee(db, employee_id, user.id).await?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::EmployeeCreated,
            entity_type: Some(AuditEntityType::Employee),
            entity_id: Some(created_id.to_string()),
            entity_label: Some(input.email),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/employees/{}", created_id)),
            metadata_json: Some(serde_json::json!({
                "user_id": user.id.to_string(),
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(created_id)
}
