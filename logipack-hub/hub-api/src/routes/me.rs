use axum::{Json, Router, extract::State, routing::get};
use uuid::Uuid;

use crate::{dto::me::MeResponse, error::ApiError, state::AppState};

use super::auth_sub::extract_sub;

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me_handler))
}

async fn me_handler(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
) -> Result<Json<MeResponse>, ApiError> {
    let (parts, _body) = request.into_parts();

    let sub = extract_sub(&parts, state.auth_mode)?;

    let role = core_application::users::me::get_me_role(&state.db, &sub).await?;
    let actor = crate::actor_extractor::resolve_actor_for_me(&state.db, state.auth_mode, &sub)
        .await
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let office_ids = actor
        .allowed_office_ids
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>();

    let current_office_id = if let Some(employee_id) = actor.employee_id {
        core_data::repository::employee_offices_repo::EmployeeOfficesRepo::current_office_id(
            &state.db,
            employee_id,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map(|value| value.to_string())
    } else {
        None
    };

    let current_office_name = match current_office_id
        .as_ref()
        .and_then(|office_id| Uuid::parse_str(office_id).ok())
    {
        Some(office_id) => core_data::repository::offices_repo::OfficesRepo::get_office_by_id(
            &state.db,
            office_id,
        )
        .await
        .ok()
        .flatten()
        .map(|office| office.name),
        None => None,
    };

    let employee_id = actor.employee_id.map(|id| id.to_string());

    Ok(Json(MeResponse {
        role,
        office_ids,
        current_office_id,
        current_office_name,
        employee_id,
    }))
}
