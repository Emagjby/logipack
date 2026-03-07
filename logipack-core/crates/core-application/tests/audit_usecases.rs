use core_application::actor::ActorContext;
use core_application::offices::create::{CreateOffice, CreateOfficeError, create_office};
use core_application::roles::Role;
use core_application::shipments::create::{CreateShipment, create_shipment};
use core_data::entity::{audit_events, clients, employees, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Set, Statement,
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

    for table in tables {
        db.execute(Statement::from_string(
            DbBackend::Postgres,
            format!("DELETE FROM {}", table),
        ))
        .await
        .unwrap();
    }
}

async fn seed_user(db: &DatabaseConnection, tag: &str) -> Uuid {
    let id = Uuid::new_v4();

    users::ActiveModel {
        id: Set(id),
        name: Set(format!("{} user", tag)),
        email: Set(Some(format!("{}+{}@test.com", tag, id))),
        password_hash: Set(Some("x".into())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    id
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

async fn admin_actor(db: &DatabaseConnection) -> ActorContext {
    let user_id = seed_user(db, "admin").await;

    ActorContext {
        user_id,
        sub: "admin".into(),
        roles: vec![Role::Admin],
        employee_id: None,
        allowed_office_ids: vec![],
    }
}

async fn employee_actor(db: &DatabaseConnection) -> ActorContext {
    let user_id = seed_user(db, "employee").await;
    let employee_id = seed_employee(db, user_id).await;

    ActorContext {
        user_id,
        sub: "employee".into(),
        roles: vec![Role::Employee],
        employee_id: Some(employee_id),
        allowed_office_ids: vec![],
    }
}

#[tokio::test]
async fn successful_usecase_emits_audit_event() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = create_office(
        &db,
        &admin,
        CreateOffice {
            name: "Audit Office".into(),
            city: "Sofia".into(),
            address: "1 Audit Street".into(),
        },
    )
    .await
    .unwrap();

    let events = audit_events::Entity::find()
        .filter(audit_events::Column::EntityId.eq(office_id.to_string()))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action_key, "office.created");
}

#[tokio::test]
async fn forbidden_usecase_does_not_emit_audit_event() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let result = create_office(
        &db,
        &employee,
        CreateOffice {
            name: "Forbidden Office".into(),
            city: "Sofia".into(),
            address: "2 Audit Street".into(),
        },
    )
    .await;

    assert!(matches!(result, Err(CreateOfficeError::Forbidden)));

    let events = audit_events::Entity::find().all(&db).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn shipment_create_emits_shipment_audit_event() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let client_id = seed_client(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id,
            current_office_id: None,
            notes: Some("created for audit".into()),
        },
    )
    .await
    .unwrap();

    let event = audit_events::Entity::find()
        .filter(audit_events::Column::EntityId.eq(shipment_id.to_string()))
        .one(&db)
        .await
        .unwrap()
        .expect("shipment audit row");

    assert_eq!(event.action_key, "shipment.created");
}
