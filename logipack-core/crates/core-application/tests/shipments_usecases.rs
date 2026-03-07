use core_application::roles::Role;
use core_application::shipments::change_status::change_status;
use core_application::shipments::create::{CreateShipment, create_shipment};
use core_application::shipments::timeline::read_timeline;
use core_application::{actor::ActorContext, shipments::change_status::ChangeStatus};
use core_data::entity::{clients, employee_offices, employees, offices, users};
use core_domain::shipment::ShipmentStatus;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    PaginatorTrait, QueryFilter, Set, Statement,
};
use test_infra::test_db;
use uuid::Uuid;

async fn cleanup(db: &DatabaseConnection) {
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
        "packages",
        "streams",
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

async fn seed_client(db: &DatabaseConnection) -> Uuid {
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

async fn seed_office(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    offices::ActiveModel {
        id: Set(id),
        name: Set("Office".into()),
        city: Set("City".into()),
        address: Set("Address".into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

async fn seed_user(db: &DatabaseConnection, user_type: Option<String>) -> Uuid {
    let id = Uuid::new_v4();
    let email = match user_type {
        Some(t) => format!("{}+{}@test.com", t, id),
        None => format!("{}+{}@test.com", "user_any", id),
    };

    users::ActiveModel {
        id: Set(id),
        name: Set("Test User".into()),
        email: Set(Some(email)),
        password_hash: Set(Some("x".into())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

async fn seed_employee(db: &DatabaseConnection, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();

    employees::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

async fn allow_employee_office(db: &DatabaseConnection, employee_id: Uuid, office_id: Uuid) {
    employee_offices::ActiveModel {
        employee_id: Set(employee_id),
        office_id: Set(office_id),
    }
    .insert(db)
    .await
    .unwrap();
}

async fn admin_actor(db: &DatabaseConnection) -> ActorContext {
    let user_id = seed_user(db, Some("admin".to_string())).await;

    ActorContext {
        user_id,
        sub: "admin".into(),
        roles: vec![Role::Admin],
        employee_id: None,
        allowed_office_ids: vec![],
    }
}

async fn employee_actor(db: &DatabaseConnection, allowed_office_ids: Vec<Uuid>) -> ActorContext {
    let user_id = seed_user(db, Some("employee".to_string())).await;
    let employee_id = seed_employee(db, user_id).await;

    for office_id in &allowed_office_ids {
        allow_employee_office(db, employee_id, *office_id).await;
    }

    ActorContext {
        user_id,
        sub: "employee".into(),
        roles: vec![Role::Employee],
        employee_id: Some(employee_id),
        allowed_office_ids,
    }
}

#[tokio::test]
async fn admin_can_change_status() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn employee_can_send_in_transit_to_other_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office1]).await;

    change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Processed,
            to_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::InTransit,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn employee_can_change_statuts_inside_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office]).await;

    change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn employee_cannot_set_non_in_transit_to_other_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office1]).await;

    let err = change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(
        err,
        core_application::shipments::change_status::ChangeStatusError::Forbidden
    ));
}

#[tokio::test]
async fn office_hop_only_allowed_when_in_transit() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    // Move forward to Processed
    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Processed,
            to_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    // illegal office hop
    let err = change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Delivered,
            to_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(
        err,
        core_application::shipments::change_status::ChangeStatusError::Domain(_)
    ));

    // legal office hop
    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::InTransit,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn admin_can_create_shipment() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    // snapshot exists
    let snap = core_data::entity::shipments::Entity::find_by_id(shipment_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(snap.current_status, "NEW");
    assert_eq!(snap.current_office_id, Some(office));
}

#[tokio::test]
async fn employee_can_create_shipment_in_allowed_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let employee = employee_actor(&db, vec![office]).await;

    let shipment_id = create_shipment(
        &db,
        &employee,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    // snapshot exists
    let snap = core_data::entity::shipments::Entity::find_by_id(shipment_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(snap.current_office_id, Some(office));
}

#[tokio::test]
async fn employee_cannot_create_shipment_outside_allowed_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let allowed_office = seed_office(&db).await;
    let forbidden_office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let employee = employee_actor(&db, vec![allowed_office]).await;

    let err = create_shipment(
        &db,
        &employee,
        CreateShipment {
            client_id: client,
            current_office_id: Some(forbidden_office),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(
        err,
        core_application::shipments::create::CreateShipmentError::Forbidden
    ));

    let count = core_data::entity::shipments::Entity::find()
        .count(&db)
        .await
        .unwrap();

    assert_eq!(count, 0);
}

#[tokio::test]
async fn create_shipment_creates_history_and_stream() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: Some("hello".into()),
        },
    )
    .await
    .unwrap();

    // history exists
    let history = core_data::entity::shipment_status_history::Entity::find()
        .filter(core_data::entity::shipment_status_history::Column::ShipmentId.eq(shipment_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].to_status, "NEW");

    // stream exists
    let stream = core_eventstore::schema::streams::Entity::find_by_id(shipment_id)
        .one(&db)
        .await
        .unwrap();

    assert!(stream.is_some());

    let packages = core_eventstore::schema::packages::Entity::find()
        .filter(core_eventstore::schema::packages::Column::StreamId.eq(shipment_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].seq, 1); // stream init
    assert_eq!(packages[1].seq, 2); // real event
}

#[tokio::test]
async fn timeline_contains_metadata_and_domain_events_in_order() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Processed,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    let timeline = read_timeline(&db, shipment_id).await.unwrap();

    assert_eq!(timeline.len(), 4);

    assert_eq!(timeline[0].seq, 1); // metadata
    assert_eq!(timeline[0].event_type, "shipment");

    assert_eq!(timeline[1].event_type, "ShipmentCreated");
    assert_eq!(timeline[2].event_type, "StatusChanged");
    assert_eq!(timeline[3].event_type, "StatusChanged");
}

#[tokio::test]
async fn timeline_is_strictly_ordered() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    let timeline = read_timeline(&db, shipment_id).await.unwrap();

    for i in 1..timeline.len() {
        assert!(timeline[i].seq >= timeline[i - 1].seq);
    }
}

#[tokio::test]
async fn timeline_values_are_decodable() {
    let db = test_db().await;
    cleanup(&db).await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    let timeline = read_timeline(&db, shipment_id).await.unwrap();

    for item in timeline {
        let _ = item.value;
    }
}

#[tokio::test]
async fn forbidden_non_in_transit_change_does_not_append_events() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office1]).await;

    let _ = change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    let packages = core_eventstore::schema::packages::Entity::find()
        .filter(core_eventstore::schema::packages::Column::StreamId.eq(shipment_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(packages.len(), 2);
}

#[tokio::test]
async fn forbidden_non_in_transit_change_does_not_mutate_snapshot() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office1]).await;

    let _ = change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    let snap = core_data::entity::shipments::Entity::find_by_id(shipment_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(snap.current_status, "NEW");
    assert_eq!(snap.current_office_id, Some(office1));
}

#[tokio::test]
async fn forbidden_non_in_transit_change_does_not_write_history() {
    let db = test_db().await;
    cleanup(&db).await;

    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;
    let client = seed_client(&db).await;

    let admin = admin_actor(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee = employee_actor(&db, vec![office1]).await;

    let _ = change_status(
        &db,
        &employee,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap_err();

    let history = core_data::entity::shipment_status_history::Entity::find()
        .filter(core_data::entity::shipment_status_history::Column::ShipmentId.eq(shipment_id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(history.len(), 1);
}
