use core_application::actor::ActorContext;
use core_application::offices::create::{CreateOffice, CreateOfficeError, create_office};
use core_application::offices::delete::{DeleteOfficeError, delete_office};
use core_application::offices::get::{GetOfficeError, get_office};
use core_application::offices::list::ListOfficesError;
use core_application::offices::update::{UpdateOffice, UpdateOfficeError, update_office};
use core_application::roles::Role;
use core_data::entity::{employees, offices, users};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, Set, Statement};
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

async fn seed_office(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    offices::ActiveModel {
        id: Set(id),
        name: Set("Main Office".to_string()),
        city: Set("Sofia".to_string()),
        address: Set("1 Test Street".to_string()),
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

async fn employee_actor(db: &DatabaseConnection) -> ActorContext {
    let user_id = seed_user(db, Some("employee".to_string())).await;
    let employee_id = seed_employee(db, user_id).await;

    ActorContext {
        user_id,
        sub: "employee".into(),
        roles: vec![Role::Employee],
        employee_id: Some(employee_id),
        allowed_office_ids: vec![],
    }
}

async fn no_role_actor(db: &DatabaseConnection) -> ActorContext {
    let user_id = seed_user(db, None).await;

    ActorContext {
        user_id,
        sub: "".into(),
        roles: vec![],
        employee_id: None,
        allowed_office_ids: vec![],
    }
}

/* -------------------- */
/* Create Office tests */
/* -------------------- */

#[tokio::test]
async fn admin_can_create_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = create_office(
        &db,
        &admin,
        CreateOffice {
            name: "Main Office".to_string(),
            city: "Sofia".to_string(),
            address: "1 Test Street".to_string(),
        },
    )
    .await
    .unwrap();

    let result = get_office(&db, &admin, office_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "Main Office".to_string());
    assert_eq!(result.city, "Sofia".to_string());
    assert_eq!(result.address, "1 Test Street".to_string());
}

#[tokio::test]
async fn employee_cannot_create_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let result = create_office(
        &db,
        &employee,
        CreateOffice {
            name: "Main Office".to_string(),
            city: "Sofia".to_string(),
            address: "1 Test Street".to_string(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_create_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let result = create_office(
        &db,
        &user,
        CreateOffice {
            name: "Main Office".to_string(),
            city: "Sofia".to_string(),
            address: "1 Test Street".to_string(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateOfficeError::Forbidden));
}

#[tokio::test]
async fn invalid_office_data_cannot_create_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let invalid_name = create_office(
        &db,
        &admin,
        CreateOffice {
            name: "".to_string(),
            city: "Sofia".to_string(),
            address: "1 Test Street".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_name, CreateOfficeError::Validation(_)));

    let invalid_city = create_office(
        &db,
        &admin,
        CreateOffice {
            name: "Main Office".to_string(),
            city: "".to_string(),
            address: "1 Test Street".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_city, CreateOfficeError::Validation(_)));

    let invalid_address = create_office(
        &db,
        &admin,
        CreateOffice {
            name: "Main Office".to_string(),
            city: "Sofia".to_string(),
            address: "".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_address, CreateOfficeError::Validation(_)));
}

/* -------------------- */
/* Update Offices tests */
/* -------------------- */

#[tokio::test]
async fn admin_can_update_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = seed_office(&db).await;

    let updated_id = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: office_id,
            name: Some("Updated Office".to_string()),
            city: None,
            address: None,
        },
    )
    .await
    .unwrap();

    let result = get_office(&db, &admin, updated_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "Updated Office".to_string());
    assert_eq!(result.city, "Sofia".to_string());
    assert_eq!(result.address, "1 Test Street".to_string());
}

#[tokio::test]
async fn employee_cannot_update_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = update_office(
        &db,
        &employee,
        UpdateOffice {
            id: office_id,
            name: Some("Updated Office".to_string()),
            city: None,
            address: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_update_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = update_office(
        &db,
        &user,
        UpdateOffice {
            id: office_id,
            name: Some("Updated Office".to_string()),
            city: None,
            address: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateOfficeError::Forbidden));
}

#[tokio::test]
async fn invalid_office_data_cannot_update_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let office_id = seed_office(&db).await;

    let invalid_name = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: office_id,
            name: Some("".to_string()),
            city: None,
            address: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_name, UpdateOfficeError::Validation(_)));

    let invalid_city = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: office_id,
            name: None,
            city: Some("".to_string()),
            address: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_city, UpdateOfficeError::Validation(_)));

    let invalid_address = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: office_id,
            name: None,
            city: None,
            address: Some("".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_address, UpdateOfficeError::Validation(_)));
}

#[tokio::test]
async fn updating_nonexistent_office_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: Uuid::new_v4(),
            name: Some("Updated Office".to_string()),
            city: None,
            address: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateOfficeError::NotFound));
}

#[tokio::test]
async fn updating_deleted_office_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = seed_office(&db).await;

    delete_office(&db, &admin, office_id).await.unwrap();

    let result = update_office(
        &db,
        &admin,
        UpdateOffice {
            id: office_id,
            name: None,
            city: None,
            address: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateOfficeError::NotFound));
}

/* -------------------- */
/* Delete Office tests */
/* -------------------- */

#[tokio::test]
async fn admin_can_delete_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = seed_office(&db).await;

    delete_office(&db, &admin, office_id).await.unwrap();

    let result = get_office(&db, &admin, office_id).await.unwrap_err();
    assert!(matches!(result, GetOfficeError::NotFound));
}

#[tokio::test]
async fn employee_cannot_delete_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = delete_office(&db, &employee, office_id).await.unwrap_err();

    assert!(matches!(result, DeleteOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_delete_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = delete_office(&db, &user, office_id).await.unwrap_err();

    assert!(matches!(result, DeleteOfficeError::Forbidden));
}

#[tokio::test]
async fn deleting_nonexistent_office_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = delete_office(&db, &admin, Uuid::new_v4())
        .await
        .unwrap_err();

    assert!(matches!(result, DeleteOfficeError::NotFound));
}

#[tokio::test]
async fn deleting_already_deleted_office_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = seed_office(&db).await;

    delete_office(&db, &admin, office_id).await.unwrap();

    let result = delete_office(&db, &admin, office_id).await.unwrap_err();

    assert!(matches!(result, DeleteOfficeError::NotFound));
}

/* ---------------- */
/* Get Office tests */
/* ---------------- */

#[tokio::test]
async fn admin_can_get_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = get_office(&db, &admin, office_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "Main Office".to_string());
    assert_eq!(result.city, "Sofia".to_string());
    assert_eq!(result.address, "1 Test Street".to_string());
}

#[tokio::test]
async fn employee_cannot_get_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = get_office(&db, &employee, office_id).await.unwrap_err();

    assert!(matches!(result, GetOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_get_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let office_id = seed_office(&db).await;

    let result = get_office(&db, &user, office_id).await.unwrap_err();

    assert!(matches!(result, GetOfficeError::Forbidden));
}

#[tokio::test]
async fn getting_nonexistent_office_returns_none() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = get_office(&db, &admin, Uuid::new_v4()).await.unwrap_err();

    assert!(matches!(result, GetOfficeError::NotFound));
}

/* ----------------- */
/* List Offices test */
/* ----------------- */

#[tokio::test]
async fn admin_can_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    // Seed multiple offices
    for i in 0..5 {
        create_office(
            &db,
            &admin,
            CreateOffice {
                name: format!("Office {}", i),
                city: "Sofia".to_string(),
                address: format!("{} Test Street", i),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::offices::list::list_offices(&db, &admin)
        .await
        .unwrap();

    assert_eq!(result.len(), 5);
}

#[tokio::test]
async fn employee_cannot_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let admin = admin_actor(&db).await;

    // Seed multiple offices
    for i in 0..5 {
        create_office(
            &db,
            &admin,
            CreateOffice {
                name: format!("Office {}", i),
                city: "Sofia".to_string(),
                address: format!("{} Test Street", i),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::offices::list::list_offices(&db, &employee)
        .await
        .unwrap_err();

    assert!(matches!(result, ListOfficesError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let admin = admin_actor(&db).await;

    // Seed multiple offices
    for i in 0..5 {
        create_office(
            &db,
            &admin,
            CreateOffice {
                name: format!("Office {}", i),
                city: "Sofia".to_string(),
                address: format!("{} Test Street", i),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::offices::list::list_offices(&db, &user)
        .await
        .unwrap_err();

    assert!(matches!(result, ListOfficesError::Forbidden));
}
