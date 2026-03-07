use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Set, Statement,
};
use uuid::Uuid;

use core_data::entity::{clients, shipment_status_history};
use core_data::repository::shipments_repo::ShipmentsRepo;
use core_domain::shipment::ShipmentStatus;
use test_infra::test_db;

pub async fn seed_client(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    clients::ActiveModel {
        id: Set(id),
        name: Set("Test Client".into()),
        phone: Set(None),
        email: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_user(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    core_data::entity::users::ActiveModel {
        id: Set(id),
        name: Set("Test User".into()),
        email: Set(Some(format!("user+{id}@test.com"))),
        password_hash: Set(Some("x".into())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn cleanup_core_data(db: &DatabaseConnection) {
    let tables = [
        "audit_events",
        "shipment_status_history",
        "shipments",
        "employee_offices",
        "employees",
        "user_roles",
        "users",
        "clients",
        "roles",
        "offices",
    ];

    for t in tables {
        db.execute(Statement::from_string(
            DbBackend::Postgres,
            format!("DELETE FROM {}", t),
        ))
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn insert_snapshot_creates_row() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let shipment_id = Uuid::new_v4();
    let client_id = seed_client(&db).await;

    ShipmentsRepo::insert_snapshot(&db, shipment_id, client_id, ShipmentStatus::New, None)
        .await
        .unwrap();

    let snap = ShipmentsRepo::get_snapshot(&db, shipment_id).await.unwrap();

    assert_eq!(snap.current_status, "NEW");
}

#[tokio::test]
async fn insert_history_creates_row() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let shipment_id = Uuid::new_v4();
    let client_id = seed_client(&db).await;

    ShipmentsRepo::insert_snapshot(&db, shipment_id, client_id, ShipmentStatus::New, None)
        .await
        .unwrap();

    ShipmentsRepo::insert_history(
        &db,
        shipment_id,
        None,
        ShipmentStatus::New,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let rows = shipment_status_history::Entity::find()
        .filter(shipment_status_history::Column::ShipmentId.eq(shipment_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn update_snapshot_updates_status() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let shipment_id = Uuid::new_v4();
    let client_id = seed_client(&db).await;

    ShipmentsRepo::insert_snapshot(&db, shipment_id, client_id, ShipmentStatus::New, None)
        .await
        .unwrap();

    ShipmentsRepo::update_snapshot_status(&db, shipment_id, ShipmentStatus::Accepted, None)
        .await
        .unwrap();

    let snap = ShipmentsRepo::get_snapshot(&db, shipment_id).await.unwrap();

    assert_eq!(snap.current_status, "ACCEPTED");
}
