use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, TransactionTrait,
};
use thiserror::Error;
use uuid::Uuid;

use crate::entity::{employee_offices, employees, roles, user_roles, users};

#[derive(Debug, Error)]
pub enum EmployeeError {
    #[error("db error: {0}")]
    EmployeeDbError(#[from] DbErr),
    #[error("employee not found")]
    RecordNotFound,
    #[error("related user not found for employee")]
    RelatedUserNotFound,
}

#[derive(Debug, Clone)]
pub struct EmployeeWithUserAndOffices {
    pub employee: employees::Model,
    pub user: users::Model,
    pub office_ids: Vec<Uuid>,
}

pub struct EmployeesRepo;

impl EmployeesRepo {
    /// Creates a new employee
    pub async fn create_employee(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, EmployeeError> {
        let txn = db.begin().await?;

        let existing = employees::Entity::find()
            .filter(employees::Column::UserId.eq(user_id))
            .one(&txn)
            .await?;

        if let Some(employee) = existing {
            if employee.deleted_at.is_some() {
                let mut model = employee.into_active_model();
                model.deleted_at = Set(None);
                model.updated_at = Set(chrono::Utc::now().into());
                let updated = model.update(&txn).await?;
                ensure_employee_role(&txn, user_id).await?;
                txn.commit().await?;
                return Ok(updated.id);
            }

            ensure_employee_role(&txn, user_id).await?;
            txn.commit().await?;
            return Ok(employee.id);
        }

        let model = employees::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            deleted_at: Set(None),
        };

        model.insert(&txn).await?;
        ensure_employee_role(&txn, user_id).await?;
        txn.commit().await?;
        Ok(id)
    }

    /// Gets employee by id
    pub async fn get_employee_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<EmployeeWithUserAndOffices, EmployeeError> {
        let retrieved = employees::Entity::find_by_id(id)
            .filter(employees::Column::DeletedAt.is_null())
            .find_also_related(users::Entity)
            .one(db)
            .await?;

        let employee_offices = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(id))
            .all(db)
            .await?;

        let user = retrieved
            .and_then(|(employee, user_opt)| user_opt.map(|user| (employee, user)))
            .ok_or(EmployeeError::RecordNotFound)?;

        let result = EmployeeWithUserAndOffices {
            employee: user.0,
            user: user.1,
            office_ids: employee_offices
                .into_iter()
                .map(|eo| eo.office_id)
                .collect::<Vec<_>>(),
        };

        Ok(result)
    }

    /// Lists all employees
    pub async fn list_employees(
        db: &DatabaseConnection,
    ) -> Result<Vec<EmployeeWithUserAndOffices>, EmployeeError> {
        let retrieved = employees::Entity::find()
            .filter(employees::Column::DeletedAt.is_null())
            .find_also_related(users::Entity)
            .all(db)
            .await?;

        let employee_ids = retrieved.iter().map(|(emp, _)| emp.id).collect::<Vec<_>>();
        let employees_offices = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.is_in(employee_ids))
            .all(db)
            .await?;

        let user_vec = retrieved
            .into_iter()
            .filter_map(|(employee, user_opt)| user_opt.map(|user| (employee, user)))
            .collect::<Vec<_>>();

        let result = user_vec
            .into_iter()
            .map(|(employee, user)| {
                let office_ids = employees_offices
                    .iter()
                    .filter(|eo| eo.employee_id == employee.id)
                    .map(|eo| eo.office_id)
                    .collect::<Vec<_>>();

                EmployeeWithUserAndOffices {
                    employee,
                    user,
                    office_ids,
                }
            })
            .collect::<Vec<_>>();

        Ok(result)
    }

    /// Updates an employee's `updated_at` timestamp (touch).
    ///
    /// Currently no user-visible fields are modified — this acts as a
    /// timestamp bump only. Extend when mutable employee fields are added.
    pub async fn update_employee(db: &DatabaseConnection, id: Uuid) -> Result<(), EmployeeError> {
        let mut model = employees::Entity::find_by_id(id)
            .filter(employees::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or(EmployeeError::RecordNotFound)?
            .into_active_model();

        model.updated_at = Set(chrono::Utc::now().into());

        model.update(db).await?;
        Ok(())
    }

    /// Soft deletes an employee by id
    pub async fn delete_employee(db: &DatabaseConnection, id: Uuid) -> Result<(), EmployeeError> {
        let txn = db.begin().await?;

        let employee = employees::Entity::find_by_id(id)
            .filter(employees::Column::DeletedAt.is_null())
            .one(&txn)
            .await?
            .ok_or(EmployeeError::RecordNotFound)?;

        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(employee.user_id))
            .exec(&txn)
            .await?;

        employees::Entity::update_many()
            .col_expr(
                employees::Column::DeletedAt,
                sea_orm::sea_query::Expr::cust("NOW()"),
            )
            .filter(employees::Column::Id.eq(id))
            .filter(employees::Column::DeletedAt.is_null())
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}

async fn ensure_employee_role(
    db: &impl sea_orm::ConnectionTrait,
    user_id: Uuid,
) -> Result<(), EmployeeError> {
    let role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("employee"))
        .one(db)
        .await?
    {
        Some(r) => r,
        None => {
            let role_id = Uuid::new_v4();
            match (roles::ActiveModel {
                id: Set(role_id),
                name: Set("employee".into()),
            })
            .insert(db)
            .await
            {
                Ok(r) => r,
                Err(_) => roles::Entity::find()
                    .filter(roles::Column::Name.eq("employee"))
                    .one(db)
                    .await?
                    .ok_or(EmployeeError::RelatedUserNotFound)?,
            }
        }
    };

    let existing = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .filter(user_roles::Column::RoleId.eq(role.id))
        .one(db)
        .await?;

    if existing.is_none() {
        let link = user_roles::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(role.id),
        };
        link.insert(db).await?;
    }

    Ok(())
}
