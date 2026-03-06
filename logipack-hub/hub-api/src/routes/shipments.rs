use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{
    dto::shipments::{
        ChangeStatusRequest, CreateShipmentRequest, CreateShipmentResponse, ShipmentDetail,
        ShipmentListItem, TimelineItem,
    },
    error::ApiError,
    policy,
    state::AppState,
};

use core_application::{
    actor::ActorContext,
    shipments::{
        change_status::{ChangeStatus, change_status},
        create::{CreateShipment, create_shipment},
        get as shipments_get, list as shipments_list,
        timeline::read_timeline,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shipments))
        .route("/:id", get(get_shipment))
        .route("/", post(create_shipment_handler))
        .route("/:id/status", post(change_status_handler))
        .route("/:id/timeline", get(get_timeline_handler))
}

/// List all shipments
async fn list_shipments(
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<Vec<ShipmentListItem>>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let rows = shipments_list::list_shipments(&state.db, &actor).await?;
    let result = rows.into_iter().map(ShipmentListItem::from).collect();
    Ok(Json(result))
}

/// Get shipment by id
async fn get_shipment(
    Path(id): Path<String>,
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<ShipmentDetail>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let shipment_id = id
        .parse()
        .map_err(|_e| ApiError::bad_request("invalid_shipment_id", "Invalid shipment id"))?;
    let row = shipments_get::get_shipment(&state.db, &actor, shipment_id).await?;
    let result = ShipmentDetail::from(row);
    Ok(Json(result))
}

async fn create_shipment_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Json(req): Json<CreateShipmentRequest>,
) -> Result<Json<CreateShipmentResponse>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let id = create_shipment(
        &state.db,
        &actor,
        CreateShipment {
            client_id: req.client_id,
            current_office_id: req.current_office_id,
            notes: req.notes,
        },
    )
    .await?;

    Ok(Json(CreateShipmentResponse { shipment_id: id }))
}

async fn change_status_handler(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    actor: ActorContext,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    change_status(
        &state.db,
        &actor,
        ChangeStatus {
            shipment_id: id,
            to_status: req.to_status,
            to_office_id: req.to_office_id,
            notes: req.notes,
        },
    )
    .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn get_timeline_handler(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    actor: ActorContext,
) -> Result<Json<Vec<TimelineItem>>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let _ = shipments_get::get_shipment(&state.db, &actor, id).await?;

    let rows = read_timeline(&state.db, id).await?;
    let result = rows
        .into_iter()
        .filter(|event| {
            !(event.seq == 1 && event.event_type.trim().eq_ignore_ascii_case("shipment"))
        })
        .map(TimelineItem::from)
        .collect();

    Ok(Json(result))
}
