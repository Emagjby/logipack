use core_data::repository::employees_repo::{self, EmployeeError, EmployeeWithUserAndOffices};
use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::actor::ActorContext;

#[derive(Debug, Error)]
pub enum GetEmployeeError {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    EmployeeError(#[from] EmployeeError),
}

pub async fn get_employee(
    db: &DatabaseConnection,
    actor: &ActorContext,
    id: uuid::Uuid,
) -> Result<EmployeeWithUserAndOffices, GetEmployeeError> {
    // Only admin can get employees
    if !actor.is_admin() {
        return Err(GetEmployeeError::Forbidden);
    }

    let result = employees_repo::EmployeesRepo::get_employee_by_id(db, id)
        .await
        .map_err(|e| match e {
            EmployeeError::RecordNotFound => GetEmployeeError::NotFound,
            other => GetEmployeeError::EmployeeError(other),
        })?;

    Ok(result)
}
