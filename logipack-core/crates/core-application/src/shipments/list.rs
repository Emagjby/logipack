use crate::actor::ActorContext;
use crate::roles::Role;
use core_data::{
    entity::shipments,
    repository::shipments_repo::{ShipmentSnapshotError, ShipmentsRepo},
};
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

pub async fn list_shipments(
    db: &DatabaseConnection,
    actor: &ActorContext,
) -> Result<Vec<shipments::Model>, ShipmentSnapshotError> {
    if actor.roles.contains(&Role::Admin) {
        return ShipmentsRepo::list_snapshots(db).await;
    }

    let allowed_ids = actor.allowed_office_ids.clone();
    if allowed_ids.is_empty() {
        return Ok(vec![]);
    }

    let rows = shipments::Entity::find()
        .filter(shipments::Column::CurrentOfficeId.is_in(allowed_ids.clone()))
        .order_by_desc(shipments::Column::CreatedAt)
        .all(db)
        .await
        .map_err(ShipmentSnapshotError::from)?;

    Ok(rows)
}
