use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use core_application::actor::ActorContext;

use crate::{
    dto::employee_offices::{AssignOfficeRequest, ListEmployeeOfficesResponse},
    dto::employees::{EmployeeDto, UserDto},
    dto::offices::OfficeDto,
    error::ApiError,
    policy,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_employee_offices_handler))
        .route("/", post(assign_office_handler))
        .route("/:officeId", delete(remove_office_handler))
}

async fn assign_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(employee_id): Path<String>,
    Json(request): Json<AssignOfficeRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let employee_uuid = employee_id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_employee_id", "Employee ID must be a valid UUID")
    })?;

    let office_uuid = request.office_id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_office_id", "Office ID must be a valid UUID")
    })?;

    let input = core_application::employee_offices::assign::AssignOffice {
        employee_id: employee_uuid,
        office_id: office_uuid,
    };

    core_application::employee_offices::assign::assign_office(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::employee_offices::assign::AssignOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::employee_offices::assign::AssignOfficeError::EmployeeNotFound => {
                ApiError::not_found("employee_not_found", "Employee not found")
            }
            core_application::employee_offices::assign::AssignOfficeError::OfficeNotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::employee_offices::assign::AssignOfficeError::AlreadyAssigned => {
                ApiError::conflict("assignment_exists", "Employee already assigned to office")
            }
            core_application::employee_offices::assign::AssignOfficeError::AssignError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    Ok(axum::http::StatusCode::OK)
}

async fn remove_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path((employee_id, office_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let employee_uuid = employee_id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_employee_id", "Employee ID must be a valid UUID")
    })?;

    let office_uuid = office_id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_office_id", "Office ID must be a valid UUID")
    })?;

    let input = core_application::employee_offices::remove::RemoveOffice {
        employee_id: employee_uuid,
        office_id: office_uuid,
    };

    core_application::employee_offices::remove::remove_office(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::employee_offices::remove::RemoveOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::employee_offices::remove::RemoveOfficeError::EmployeeNotFound => {
                ApiError::not_found("employee_not_found", "Employee not found")
            }
            core_application::employee_offices::remove::RemoveOfficeError::OfficeNotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::employee_offices::remove::RemoveOfficeError::RemoveError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn list_employee_offices_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(employee_id): Path<String>,
) -> Result<Json<ListEmployeeOfficesResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let employee_uuid = employee_id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_employee_id", "Employee ID must be a valid UUID")
    })?;

    let out = core_application::employees::get::get_employee(&state.db, &actor, employee_uuid)
        .await
        .map_err(|e| match e {
            core_application::employees::get::GetEmployeeError::NotFound => {
                ApiError::not_found("employee_not_found", "Employee not found")
            }
            core_application::employees::get::GetEmployeeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::employees::get::GetEmployeeError::EmployeeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let office_ids = out
        .office_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>();

    let offices = core_application::offices::list::list_offices(&state.db, &actor)
        .await
        .map_err(|e| match e {
            core_application::offices::list::ListOfficesError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::list::ListOfficesError::OfficeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let office_dtos = offices
        .into_iter()
        .map(|office| OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        })
        .collect::<Vec<_>>();

    let assigned_offices = office_dtos
        .iter()
        .filter(|office| office_ids.contains(&office.id))
        .map(|office| OfficeDto {
            id: office.id.clone(),
            name: office.name.clone(),
            city: office.city.clone(),
            address: office.address.clone(),
            updated_at: office.updated_at.clone(),
        })
        .collect::<Vec<_>>();

    let response = ListEmployeeOfficesResponse {
        employee_id: employee_uuid.to_string(),
        offices: office_dtos,
        office_ids,
        employee: Some(EmployeeDto {
            id: out.employee.id.to_string(),
            user_id: out.employee.user_id.to_string(),
            full_name: out.user.name.clone(),
            user_display_name: None,
            email: out.user.email.clone().unwrap_or_default(),
            user: Some(UserDto {
                id: out.user.id.to_string(),
                email: out.user.email.unwrap_or_default(),
                name: Some(out.user.name),
            }),
            offices: None,
            created_at: Some(out.employee.created_at.to_rfc3339()),
            updated_at: Some(out.employee.updated_at.to_rfc3339()),
            deleted_at: out.employee.deleted_at.map(|dt| dt.to_rfc3339()),
        }),
        assigned_offices: Some(assigned_offices),
    };

    Ok(Json(response))
}
