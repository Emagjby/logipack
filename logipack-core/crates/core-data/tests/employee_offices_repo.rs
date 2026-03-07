use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, Set, Statement};
use uuid::Uuid;

use core_data::entity::{employees, offices, users};
use core_data::repository::employee_offices_repo::{EmployeeOfficeError, EmployeeOfficesRepo};
use test_infra::test_db;

pub async fn seed_user(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    users::ActiveModel {
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

pub async fn seed_employee(db: &DatabaseConnection) -> Uuid {
    let user_id = seed_user(db).await;
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

pub async fn seed_office(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    offices::ActiveModel {
        id: Set(id),
        name: Set("Test Office".into()),
        city: Set("Test City".into()),
        address: Set("Test Address".into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_soft_deleted_employee(db: &DatabaseConnection) -> Uuid {
    let user_id = seed_user(db).await;
    let id = Uuid::new_v4();

    employees::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(Some(chrono::Utc::now().into())),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_soft_deleted_office(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    offices::ActiveModel {
        id: Set(id),
        name: Set("Deleted Office".into()),
        city: Set("Test City".into()),
        address: Set("Test Address".into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(Some(chrono::Utc::now().into())),
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
async fn assign_creates_row() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_office(&db).await;

    let result = EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap();

    assert!(result); // true = new row inserted

    let offices = EmployeeOfficesRepo::list_offices(&db, employee_id)
        .await
        .unwrap();
    assert_eq!(offices.len(), 1);
    assert_eq!(offices[0], office_id);
}

#[tokio::test]
async fn assign_duplicate_is_idempotent() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_office(&db).await;

    let first = EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap();
    assert!(first);

    let second = EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap();
    assert!(!second); // false = already existed

    let offices = EmployeeOfficesRepo::list_offices(&db, employee_id)
        .await
        .unwrap();
    assert_eq!(offices.len(), 1);
}

#[tokio::test]
async fn remove_deletes_row() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_office(&db).await;

    EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap();

    EmployeeOfficesRepo::remove_office(&db, employee_id, office_id)
        .await
        .unwrap();

    let offices = EmployeeOfficesRepo::list_offices(&db, employee_id)
        .await
        .unwrap();
    assert!(offices.is_empty());
}

#[tokio::test]
async fn remove_missing_is_noop() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_office(&db).await;

    // Remove without prior assign — should not error
    EmployeeOfficesRepo::remove_office(&db, employee_id, office_id)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_returns_correct_office_ids() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id_1 = seed_office(&db).await;
    let office_id_2 = seed_office(&db).await;
    let office_id_3 = seed_office(&db).await;

    EmployeeOfficesRepo::assign_office(&db, employee_id, office_id_1)
        .await
        .unwrap();
    EmployeeOfficesRepo::assign_office(&db, employee_id, office_id_2)
        .await
        .unwrap();
    EmployeeOfficesRepo::assign_office(&db, employee_id, office_id_3)
        .await
        .unwrap();

    let mut offices = EmployeeOfficesRepo::list_offices(&db, employee_id)
        .await
        .unwrap();
    offices.sort();

    let mut expected = vec![office_id_1, office_id_2, office_id_3];
    expected.sort();

    assert_eq!(offices, expected);
}

#[tokio::test]
async fn employee_not_found_on_assign() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let office_id = seed_office(&db).await;

    let result = EmployeeOfficesRepo::assign_office(&db, Uuid::new_v4(), office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn office_not_found_on_assign() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;

    let result = EmployeeOfficesRepo::assign_office(&db, employee_id, Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::OfficeNotFound));
}

#[tokio::test]
async fn employee_not_found_on_list() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let result = EmployeeOfficesRepo::list_offices(&db, Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn employee_not_found_on_remove() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let office_id = seed_office(&db).await;

    let result = EmployeeOfficesRepo::remove_office(&db, Uuid::new_v4(), office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn soft_deleted_employee_not_found() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_soft_deleted_employee(&db).await;
    let office_id = seed_office(&db).await;

    // assign
    let result = EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));

    // list
    let result = EmployeeOfficesRepo::list_offices(&db, employee_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));

    // remove
    let result = EmployeeOfficesRepo::remove_office(&db, employee_id, office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn soft_deleted_office_not_found_on_assign() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_soft_deleted_office(&db).await;

    let result = EmployeeOfficesRepo::assign_office(&db, employee_id, office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::OfficeNotFound));
}

#[tokio::test]
async fn remove_office_returns_office_not_found_when_office_missing() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;

    let result = EmployeeOfficesRepo::remove_office(&db, employee_id, Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::OfficeNotFound));
}

#[tokio::test]
async fn remove_office_returns_office_not_found_when_office_soft_deleted() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let employee_id = seed_employee(&db).await;
    let office_id = seed_soft_deleted_office(&db).await;

    let result = EmployeeOfficesRepo::remove_office(&db, employee_id, office_id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeOfficeError::OfficeNotFound));
}
