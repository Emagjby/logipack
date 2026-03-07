use chrono::Utc;
use core_data::repository::shipments_repo::ShipmentSnapshotError;
use core_data::repository::shipments_repo::ShipmentsRepo;
use core_domain::errors::TransitionError;
use core_domain::shipment::{ShipmentStatus, validate_transition};
use core_eventstore::adapter::streams::EnsureStreamError;
use sea_orm::DatabaseConnection;
use strata::value::Value;
use strata::{int, map, null, string};
use thiserror::Error;
use uuid::Uuid;

use crate::actor::ActorContext;
use crate::audit::{
    AuditActionKey, AuditEntityType, AuditError, AuditEventInput, emit_audit_event,
};

#[derive(Debug, Clone)]
pub struct ChangeStatus {
    pub shipment_id: Uuid,
    pub to_status: ShipmentStatus,
    pub to_office_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Error)]
pub enum ChangeStatusError {
    #[error("forbidden")]
    Forbidden,
    #[error("domain transition error: {0:?}")]
    Domain(#[from] TransitionError),
    #[error("snapshot error: {0}")]
    SnapshotError(#[from] ShipmentSnapshotError),
    #[error("stream error: {0}")]
    StreamError(#[from] EnsureStreamError),
    #[error("db error: {0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("eventstore error: {0}")]
    EventstoreError(#[from] core_eventstore::adapter::append::AppendError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

pub async fn change_status(
    db: &DatabaseConnection,
    actor: &ActorContext,
    input: ChangeStatus,
) -> Result<(), ChangeStatusError> {
    let snap = ShipmentsRepo::get_snapshot(db, input.shipment_id).await?;

    let from_status: ShipmentStatus = snap.current_status.parse().unwrap_or(ShipmentStatus::New);
    let current_office = snap.current_office_id;

    // employees can only write within current shipment office
    if !actor.is_admin() {
        let shipment_office = current_office.ok_or(ChangeStatusError::Forbidden)?;

        if !actor.allowed_office_ids.contains(&shipment_office) {
            return Err(ChangeStatusError::Forbidden);
        }
    }

    // employees may route IN_TRANSIT shipments to any destination office.
    // For other statuses, destination office must remain within employee scope.
    if !actor.is_admin()
        && input.to_status != ShipmentStatus::InTransit
        && let Some(to_office) = input.to_office_id
        && !actor.allowed_office_ids.contains(&to_office)
    {
        return Err(ChangeStatusError::Forbidden);
    }

    let office_changed = input.to_office_id.is_some() && input.to_office_id != current_office;
    let has_current_office = current_office.is_some();
    let has_target_office = input.to_office_id.is_some();

    validate_transition(
        from_status,
        input.to_status,
        office_changed,
        has_current_office,
        has_target_office,
    )
    .map_err(ChangeStatusError::Domain)?;

    // projection trio
    core_eventstore::adapter::streams::ensure_stream(db, input.shipment_id, "shipment").await?;

    let occured_at = Utc::now().timestamp_millis();

    let payload: Value = map! {
        "event_type" => string!("StatusChanged"),
        "shipment_id" => string!(input.shipment_id.to_string()),
        "from_status" => string!(from_status.to_string()),
        "to_status" => string!(input.to_status.to_string()),
        "actor_user_id" => string!(actor.user_id.to_string()),
        "from_office_id" => match snap.current_office_id {
            Some(office_id) => string!(office_id.to_string()),
            None => null!(),
        },
        "to_office_id" => match input.to_office_id {
            Some(office_id) => string!(office_id.to_string()),
            None => null!(),
        },
        "occured_at" => int!(occured_at),
        "notes" => match input.notes {
            Some(ref notes) => string!(notes),
            None => null!(),
        }
    };

    // immutable audit
    core_eventstore::adapter::append::append_package(
        db,
        input.shipment_id,
        "StatusChanged",
        &payload,
    )
    .await?;

    // history row
    ShipmentsRepo::insert_history(
        db,
        input.shipment_id,
        Some(from_status),
        input.to_status,
        Some(actor.user_id),
        current_office,
        input.notes.clone(),
    )
    .await?;

    // snapshot update
    // only hop office when going to IN_TRANSIT
    let new_office = if input.to_status == ShipmentStatus::InTransit
        || (from_status == ShipmentStatus::InTransit && input.to_status == ShipmentStatus::Accepted)
    {
        input.to_office_id.or(current_office)
    } else {
        None // Keep old
    };

    ShipmentsRepo::update_snapshot_status(db, input.shipment_id, input.to_status, new_office)
        .await?;

    emit_audit_event(
        db,
        actor,
        AuditEventInput {
            action_key: AuditActionKey::ShipmentStatusUpdated,
            entity_type: Some(AuditEntityType::Shipment),
            entity_id: Some(input.shipment_id.to_string()),
            entity_label: Some(format!("Shipment {}", input.shipment_id)),
            office_id: input.to_office_id.or(current_office),
            office_label: input
                .to_office_id
                .or(current_office)
                .map(|office_id| format!("Office {}", office_id)),
            target_route: Some(format!("/app/admin/shipments/{}", input.shipment_id)),
            metadata_json: Some(serde_json::json!({
                "from_status": from_status.to_string(),
                "to_status": input.to_status.to_string(),
                "from_office_id": current_office.map(|value| value.to_string()),
                "to_office_id": input.to_office_id.map(|value| value.to_string()),
                "notes": input.notes,
            })),
            request_id: None,
            occurred_at: None,
        },
    )
    .await?;

    Ok(())
}
