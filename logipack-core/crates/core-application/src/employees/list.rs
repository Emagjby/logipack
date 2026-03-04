use core_data::repository::employees_repo::{self, EmployeeError, EmployeeWithUserAndOffices};
use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::actor::ActorContext;

#[derive(Debug, Error)]
pub enum ListEmployeesError {
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    EmployeeError(#[from] EmployeeError),
}

pub async fn list_employees(
    db: &DatabaseConnection,
    actor: &ActorContext,
) -> Result<Vec<EmployeeWithUserAndOffices>, ListEmployeesError> {
    // Only admin can list employees
    if !actor.is_admin() {
        return Err(ListEmployeesError::Forbidden);
    }

    let employees = employees_repo::EmployeesRepo::list_employees(db).await?;

    Ok(employees)
}
