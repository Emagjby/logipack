use axum::{Json, Router, extract::State, routing::get};
use core_application::actor::ActorContext;
use core_data::repository::offices_repo::OfficesRepo;

use crate::{
    dto::offices::{ListOfficesResponse, OfficeDto},
    error::ApiError,
    policy,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/offices", get(list_offices_handler))
}

async fn list_offices_handler(
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<ListOfficesResponse>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let out = OfficesRepo::list_offices(&state.db)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;

    let offices = out
        .into_iter()
        .map(|office| OfficeDto {
            id: office.id.to_string(),
            name: office.name,
            city: office.city,
            address: office.address,
            updated_at: office.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListOfficesResponse { offices }))
}
