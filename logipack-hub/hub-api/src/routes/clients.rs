use axum::{Json, Router, extract::State, routing::get};
use core_application::actor::ActorContext;
use core_data::repository::clients_repo::ClientsRepo;

use crate::{
    dto::clients::{ClientDto, ListClientsResponse},
    error::ApiError,
    policy,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/clients", get(list_clients_handler))
}

async fn list_clients_handler(
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<ListClientsResponse>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let out = ClientsRepo::list_clients(&state.db)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;

    let clients = out
        .into_iter()
        .map(|client| ClientDto {
            id: client.id.to_string(),
            name: client.name,
            email: client.email,
            phone: client.phone,
            updated_at: client.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListClientsResponse { clients }))
}
