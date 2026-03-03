use core_data::repository::employees_repo::{self, EmployeeError};
use core_data::repository::users_repo::{UserError, UserRepo};
use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;

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

    Ok(created_id)
}
