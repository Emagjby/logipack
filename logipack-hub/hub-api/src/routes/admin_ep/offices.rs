use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use core_application::actor::ActorContext;

use crate::{
    dto::offices::{
        CreateOfficeRequest, CreateOfficeResponse, GetOfficeResponse, ListOfficesResponse,
        OfficeDto, UpdateOfficeRequest, UpdateOfficeResponse,
    },
    error::ApiError,
    policy,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_offices_handler))
        .route("/:id", get(get_office_handler))
        .route("/", post(create_office_handler))
        .route("/:id", put(update_office_handler))
        .route("/:id", delete(delete_office_handler))
}

async fn list_offices_handler(
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<ListOfficesResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let out = core_application::offices::list::list_offices(&state.db, &actor)
        .await
        .map_err(|e| match e {
            core_application::offices::list::ListOfficesError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::list::ListOfficesError::OfficeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let dtos: Vec<OfficeDto> = out
        .into_iter()
        .map(|office| OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        })
        .collect();

    let result = ListOfficesResponse { offices: dtos };

    Ok(Json(result))
}

async fn get_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
) -> Result<Json<GetOfficeResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if office_id is a valid UUID
    let office_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_office_id", "Office ID must be a valid UUID")
    })?;

    let out = core_application::offices::get::get_office(&state.db, &actor, office_uuid)
        .await
        .map_err(|e| match e {
            core_application::offices::get::GetOfficeError::NotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::offices::get::GetOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::get::GetOfficeError::OfficeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let office = out.ok_or_else(|| ApiError::not_found("office_not_found", "Office not found"))?;
    let result = GetOfficeResponse {
        office: OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        },
    };

    Ok(Json(result))
}

async fn create_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Json(request): Json<CreateOfficeRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateOfficeResponse>), ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let input = core_application::offices::create::CreateOffice {
        name: request.name,
        city: request.city,
        address: request.address,
    };

    let office_id = core_application::offices::create::create_office(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::offices::create::CreateOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::create::CreateOfficeError::Validation(err) => {
                ApiError::bad_request("invalid_office", err.to_string())
            }
            core_application::offices::create::CreateOfficeError::OfficeCreationError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::offices::create::CreateOfficeError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let out = core_application::offices::get::get_office(&state.db, &actor, office_id)
        .await
        .map_err(|e| match e {
            core_application::offices::get::GetOfficeError::NotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::offices::get::GetOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::get::GetOfficeError::OfficeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let office = out.ok_or_else(|| ApiError::internal("created_office_missing"))?;

    let result = CreateOfficeResponse {
        office: OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        },
    };

    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
    Json(request): Json<UpdateOfficeRequest>,
) -> Result<Json<UpdateOfficeResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if office_id is a valid UUID
    let office_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_office_id", "Office ID must be a valid UUID")
    })?;

    let input = core_application::offices::update::UpdateOffice {
        id: office_uuid,
        name: request.name,
        city: request.city,
        address: request.address,
    };

    let updated_id = core_application::offices::update::update_office(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::offices::update::UpdateOfficeError::NotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::offices::update::UpdateOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::update::UpdateOfficeError::Validation(err) => {
                ApiError::bad_request("invalid_office", err.to_string())
            }
            core_application::offices::update::UpdateOfficeError::UpdateOfficeError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::offices::update::UpdateOfficeError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let out = core_application::offices::get::get_office(&state.db, &actor, updated_id)
        .await
        .map_err(|e| match e {
            core_application::offices::get::GetOfficeError::NotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::offices::get::GetOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::get::GetOfficeError::OfficeError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let office = out.ok_or_else(|| ApiError::internal("updated_office_missing"))?;

    let result = UpdateOfficeResponse {
        office: OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        },
    };

    Ok(Json(result))
}

async fn delete_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if office_id is a valid UUID
    let office_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_office_id", "Office ID must be a valid UUID")
    })?;

    core_application::offices::delete::delete_office(&state.db, &actor, office_uuid)
        .await
        .map_err(|e| match e {
            core_application::offices::delete::DeleteOfficeError::NotFound => {
                ApiError::not_found("office_not_found", "Office not found")
            }
            core_application::offices::delete::DeleteOfficeError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::offices::delete::DeleteOfficeError::DeleteOfficeError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::offices::delete::DeleteOfficeError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
