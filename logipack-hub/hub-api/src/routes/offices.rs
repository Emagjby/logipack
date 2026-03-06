use axum::{Json, Router, extract::State, routing::get};
use core_application::actor::ActorContext;

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
