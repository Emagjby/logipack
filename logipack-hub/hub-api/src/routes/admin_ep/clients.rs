use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use core_application::actor::ActorContext;

use crate::{
    dto::clients::{
        ClientDto, CreateClientRequest, CreateClientResponse, GetClientResponse,
        ListClientsResponse, UpdateClientRequest, UpdateClientResponse,
    },
    error::ApiError,
    policy,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_clients_handler))
        .route("/:id", get(get_client_handler))
        .route("/", post(create_client_handler))
        .route("/:id", put(update_client_handler))
        .route("/:id", delete(delete_client_handler))
}

async fn list_clients_handler(
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<ListClientsResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let out = core_application::clients::list::list_clients(&state.db, &actor)
        .await
        .map_err(|e| match e {
            core_application::clients::list::ListClientsError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::list::ListClientsError::ClientError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let dtos: Vec<ClientDto> = out
        .into_iter()
        .map(|client| ClientDto {
            id: client.id.to_string(),
            name: client.name,
            phone: client.phone,
            email: client.email,
            updated_at: client.updated_at.to_rfc3339(),
        })
        .collect();

    let result = ListClientsResponse { clients: dtos };

    Ok(Json(result))
}

async fn get_client_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
) -> Result<Json<GetClientResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if client_id is a valid UUID
    let client_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_client_id", "Client ID must be a valid UUID")
    })?;

    let out = core_application::clients::get::get_client(&state.db, &actor, client_uuid)
        .await
        .map_err(|e| match e {
            core_application::clients::get::GetClientError::NotFound => {
                ApiError::not_found("client_not_found", "Client not found")
            }
            core_application::clients::get::GetClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::get::GetClientError::ClientError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let client = out.ok_or_else(|| ApiError::not_found("client_not_found", "Client not found"))?;
    let result = GetClientResponse {
        client: ClientDto {
            id: client.id.to_string(),
            name: client.name,
            phone: client.phone,
            email: client.email,
            updated_at: client.updated_at.to_rfc3339(),
        },
    };

    Ok(Json(result))
}

async fn create_client_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Json(request): Json<CreateClientRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateClientResponse>), ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let input = core_application::clients::create::CreateClient {
        name: request.name,
        phone: request.phone,
        email: request.email,
    };

    let client_id = core_application::clients::create::create_client(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::clients::create::CreateClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::create::CreateClientError::Validation(err) => {
                ApiError::bad_request("invalid_client", err.to_string())
            }
            core_application::clients::create::CreateClientError::ClientCreationError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::clients::create::CreateClientError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let out = core_application::clients::get::get_client(&state.db, &actor, client_id)
        .await
        .map_err(|e| match e {
            core_application::clients::get::GetClientError::NotFound => {
                ApiError::not_found("client_not_found", "Client not found")
            }
            core_application::clients::get::GetClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::get::GetClientError::ClientError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let client = out.ok_or_else(|| ApiError::not_found("client_not_found", "Client not found"))?;
    let result = CreateClientResponse {
        client: ClientDto {
            id: client.id.to_string(),
            name: client.name,
            phone: client.phone,
            email: client.email,
            updated_at: client.updated_at.to_rfc3339(),
        },
    };

    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update_client_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
    Json(request): Json<UpdateClientRequest>,
) -> Result<Json<UpdateClientResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if client_id is a valid UUID
    let client_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_client_id", "Client ID must be a valid UUID")
    })?;

    let input = core_application::clients::update::UpdateClient {
        id: client_uuid,
        name: request.name,
        phone: request.phone,
        email: request.email,
    };

    let updated_id = core_application::clients::update::update_client(&state.db, &actor, input)
        .await
        .map_err(|e| match e {
            core_application::clients::update::UpdateClientError::NotFound => {
                ApiError::not_found("client_not_found", "Client not found")
            }
            core_application::clients::update::UpdateClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::update::UpdateClientError::Validation(err) => {
                ApiError::bad_request("invalid_client", err.to_string())
            }
            core_application::clients::update::UpdateClientError::UpdateClientError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::clients::update::UpdateClientError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let out = core_application::clients::get::get_client(&state.db, &actor, updated_id)
        .await
        .map_err(|e| match e {
            core_application::clients::get::GetClientError::NotFound => {
                ApiError::not_found("client_not_found", "Client not found")
            }
            core_application::clients::get::GetClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::get::GetClientError::ClientError(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    let client = out.ok_or_else(|| ApiError::not_found("client_not_found", "Client not found"))?;
    let result = UpdateClientResponse {
        client: ClientDto {
            id: client.id.to_string(),
            name: client.name,
            phone: client.phone,
            email: client.email,
            updated_at: client.updated_at.to_rfc3339(),
        },
    };

    Ok(Json(result))
}

async fn delete_client_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    // check if client_id is a valid UUID
    let client_uuid = id.parse::<uuid::Uuid>().map_err(|_| {
        ApiError::bad_request("invalid_client_id", "Client ID must be a valid UUID")
    })?;

    core_application::clients::delete::delete_client(&state.db, &actor, client_uuid)
        .await
        .map_err(|e| match e {
            core_application::clients::delete::DeleteClientError::NotFound => {
                ApiError::not_found("client_not_found", "Client not found")
            }
            core_application::clients::delete::DeleteClientError::Forbidden => {
                ApiError::forbidden("access_denied", "Access denied")
            }
            core_application::clients::delete::DeleteClientError::DeleteClientError(err) => {
                ApiError::internal(err.to_string())
            }
            core_application::clients::delete::DeleteClientError::Audit(err) => {
                ApiError::internal(err.to_string())
            }
        })?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
