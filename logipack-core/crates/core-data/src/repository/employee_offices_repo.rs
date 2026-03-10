use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use thiserror::Error;
use uuid::Uuid;

use crate::entity::{employee_offices, employees, offices};

#[derive(Debug, Error)]
pub enum EmployeeOfficeError {
    #[error("db error: {0}")]
    DbError(#[from] DbErr),
    #[error("employee not found")]
    EmployeeNotFound,
    #[error("office not found")]
    OfficeNotFound,
}

pub struct EmployeeOfficesRepo;

impl EmployeeOfficesRepo {
    /// Checks if an employee exists and is not soft-deleted
    pub async fn employee_exists(
        db: &DatabaseConnection,
        employee_id: Uuid,
    ) -> Result<bool, EmployeeOfficeError> {
        let result = employees::Entity::find_by_id(employee_id)
            .filter(employees::Column::DeletedAt.is_null())
            .one(db)
            .await?;

        Ok(result.is_some())
    }

    /// Checks if an office exists and is not soft-deleted
    pub async fn office_exists(
        db: &DatabaseConnection,
        office_id: Uuid,
    ) -> Result<bool, EmployeeOfficeError> {
        let result = offices::Entity::find_by_id(office_id)
            .filter(offices::Column::DeletedAt.is_null())
            .one(db)
            .await?;

        Ok(result.is_some())
    }

    /// Assigns an office to an employee. Returns true if a new row was inserted,
    /// false if the relation already existed (idempotent).
    pub async fn assign_office(
        db: &DatabaseConnection,
        employee_id: Uuid,
        office_id: Uuid,
    ) -> Result<bool, EmployeeOfficeError> {
        if !Self::employee_exists(db, employee_id).await? {
            return Err(EmployeeOfficeError::EmployeeNotFound);
        }

        if !Self::office_exists(db, office_id).await? {
            return Err(EmployeeOfficeError::OfficeNotFound);
        }

        // Check if relation already exists
        let existing = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(employee_id))
            .filter(employee_offices::Column::OfficeId.eq(office_id))
            .one(db)
            .await?;

        if existing.is_some() {
            return Ok(false);
        }

        let model = employee_offices::ActiveModel {
            employee_id: Set(employee_id),
            office_id: Set(office_id),
        };

        model.insert(db).await?;
        Ok(true)
    }

    /// Removes an office assignment from an employee. No-op if the relation
    /// does not exist (idempotent).
    pub async fn remove_office(
        db: &DatabaseConnection,
        employee_id: Uuid,
        office_id: Uuid,
    ) -> Result<(), EmployeeOfficeError> {
        if !Self::employee_exists(db, employee_id).await? {
            return Err(EmployeeOfficeError::EmployeeNotFound);
        }

        if !Self::office_exists(db, office_id).await? {
            return Err(EmployeeOfficeError::OfficeNotFound);
        }

        employee_offices::Entity::delete_many()
            .filter(employee_offices::Column::EmployeeId.eq(employee_id))
            .filter(employee_offices::Column::OfficeId.eq(office_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Lists all office IDs assigned to an employee.
    pub async fn list_offices(
        db: &DatabaseConnection,
        employee_id: Uuid,
    ) -> Result<Vec<Uuid>, EmployeeOfficeError> {
        if !Self::employee_exists(db, employee_id).await? {
            return Err(EmployeeOfficeError::EmployeeNotFound);
        }

        let rows = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(employee_id))
            .all(db)
            .await?;

        let office_ids = rows.into_iter().map(|r| r.office_id).collect();
        Ok(office_ids)
    }

    /// Returns a deterministic current office selection for an employee.
    /// The lowest office UUID is used when multiple assignments exist.
    pub async fn current_office_id(
        db: &DatabaseConnection,
        employee_id: Uuid,
    ) -> Result<Option<Uuid>, EmployeeOfficeError> {
        if !Self::employee_exists(db, employee_id).await? {
            return Err(EmployeeOfficeError::EmployeeNotFound);
        }

        let row = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(employee_id))
            .order_by_asc(employee_offices::Column::OfficeId)
            .one(db)
            .await?;

        Ok(row.map(|value| value.office_id))
    }
}
