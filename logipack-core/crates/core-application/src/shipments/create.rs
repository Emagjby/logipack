use chrono::Utc;
use core_data::repository::shipments_repo::ShipmentsRepo;
use core_domain::shipment::ShipmentStatus;
use core_eventstore::adapter::events::append_event;
use core_eventstore::adapter::streams::ensure_stream;
use sea_orm::DatabaseConnection;
use thiserror::Error;

use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

use strata::value::Value;
use strata::{int, map, null, string};

#[derive(Debug, Clone)]
pub struct CreateShipment {
    pub client_id: Uuid,
    pub current_office_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Error)]
pub enum CreateShipmentError {
    #[error("forbidden")]
    Forbidden,
    #[error("db error: {0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("eventstore error: {0}")]
    EventstoreError(#[from] core_eventstore::adapter::append::AppendError),
    #[error("stream error: {0}")]
    EnsureStreamError(#[from] core_eventstore::adapter::streams::EnsureStreamError),
    #[error("create shipment snapshot error: {0}")]
    SnapshotError(#[from] core_data::repository::shipments_repo::ShipmentSnapshotError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn create_shipment(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: CreateShipment,
) -> Result<Uuid, CreateShipmentError> {
    // Office scope policy
    if !actor.is_admin() {
        // employee must provide current_office_id and it must be allowed
        let office_id = input
            .current_office_id
            .ok_or(CreateShipmentError::Forbidden)?;

        if !actor.allowed_office_ids.contains(&office_id) {
            return Err(CreateShipmentError::Forbidden);
        }
    }

    let shipment_id = Uuid::new_v4();
    let status = ShipmentStatus::New;

    // snapshot
    ShipmentsRepo::insert_snapshot(
        db,
        shipment_id,
        input.client_id,
        status,
        input.current_office_id,
    )
    .await?;

    // history
    ShipmentsRepo::insert_history(
        db,
        shipment_id,
        None,
        status,
        Some(actor.user_id),
        input.current_office_id,
        input.notes.clone(),
    )
    .await?;

    // ensure stream
    ensure_stream(db, shipment_id, "shipment").await?;

    // eventstore
    append_event(db, shipment_id, "shipment", &map! {}).await?;

    let occured_at = Utc::now().timestamp_millis();

    let payload: Value = map! {
        "event_type" => string!("ShipmentCreated"),
        "shipment_id" => string!(shipment_id.to_string()),
        "status" => string!(status.to_string()),
        "actor_user_id" => string!(actor.user_id.to_string()),
        "office_id" => match input.current_office_id {
            Some(office_id) => string!(office_id.to_string()),
            None => null!(),
        },
        "occured_at" => int!(occured_at),
        "notes" => match input.notes {
            Some(ref notes) => string!(notes),
            None => null!(),
        }
    };

    core_eventstore::adapter::events::append_event(db, shipment_id, "ShipmentCreated", &payload)
        .await?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::ShipmentCreated,
            entity_type: Some(AuditEntityType::Shipment),
            entity_id: Some(shipment_id.to_string()),
            entity_label: Some(format!("Shipment {}", shipment_id)),
            office_id: input.current_office_id,
            office_label: input
                .current_office_id
                .map(|office_id| format!("Office {}", office_id)),
            target_route: Some(format!("/app/admin/shipments/{}", shipment_id)),
            metadata_json: Some(serde_json::json!({
                "client_id": input.client_id.to_string(),
                "status": status.to_string(),
                "current_office_id": input.current_office_id.map(|value| value.to_string()),
                "notes": input.notes,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(shipment_id)
}
