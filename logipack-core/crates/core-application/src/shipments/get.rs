use crate::actor::ActorContext;
use crate::roles::Role;
use core_data::{
    entity::shipments,
    repository::shipments_repo::{ShipmentSnapshotError, ShipmentsRepo},
};
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use uuid::Uuid;

fn not_found() -> ShipmentSnapshotError {
    ShipmentSnapshotError::DbError(DbErr::RecordNotFound("shipment not found".into()))
}

pub async fn get_shipment(
    db: &DatabaseConnection,
    actor: &ActorContext,
    shipment_id: Uuid,
) -> Result<shipments::Model, ShipmentSnapshotError> {
    let row = ShipmentsRepo::get_snapshot(db, shipment_id).await?;

    if actor.roles.contains(&Role::Admin) {
        return Ok(row);
    }

    let allowed_ids = &actor.allowed_office_ids;
    if allowed_ids.is_empty() {
        return Err(not_found());
    }

    if row
        .current_office_id
        .is_some_and(|office_id| allowed_ids.contains(&office_id))
    {
        return Ok(row);
    }

    Err(not_found())
}
