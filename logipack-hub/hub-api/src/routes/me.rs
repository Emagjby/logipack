use axum::{Json, Router, extract::State, routing::get};
use core_data::entity::employee_offices;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
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
        employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(employee_id))
            .one(&state.db)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
            .map(|row| row.office_id.to_string())
    } else {
        None
    };

    Ok(Json(MeResponse {
        role,
        office_ids,
        current_office_id,
    }))
}
