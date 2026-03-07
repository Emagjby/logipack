use core_application::actor::ActorContext;
use core_application::clients::create::{CreateClient, CreateClientError, create_client};
use core_application::clients::delete::{DeleteClientError, delete_client};
use core_application::clients::get::{GetClientError, get_client};
use core_application::clients::list::ListClientsError;
use core_application::clients::update::{UpdateClient, UpdateClientError, update_client};
use core_application::roles::Role;
use core_data::entity::{clients, employees, users};
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

async fn seed_client(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();

    clients::ActiveModel {
        id: Set(id),
        name: Set("John Doe".to_string()),
        phone: Set(Some("+359123456".to_string())),
        email: Set(Some("email@example.com".to_string())),
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

/* ------------------- */
/* Create Client tests */
/* ------------------- */

#[tokio::test]
async fn admin_can_create_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = create_client(
        &db,
        &admin,
        CreateClient {
            name: "John Doe".to_string(),
            phone: Some("+359123456".to_string()),
            email: Some("email@example.com".to_string()),
        },
    )
    .await
    .unwrap();

    let result = get_client(&db, &admin, client_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "John Doe".to_string());
    assert_eq!(result.phone, Some("+359123456".to_string()));
    assert_eq!(result.email, Some("email@example.com".to_string()));
}

#[tokio::test]
async fn employee_cannot_create_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let result = create_client(
        &db,
        &employee,
        CreateClient {
            name: "John Doe".to_string(),
            phone: Some("+359123456".to_string()),
            email: Some("email@example.com".to_string()),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateClientError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_create_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let result = create_client(
        &db,
        &user,
        CreateClient {
            name: "John Doe".to_string(),
            phone: Some("+359123456".to_string()),
            email: Some("email@example.com".to_string()),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateClientError::Forbidden));
}

#[tokio::test]
async fn invalid_client_data_cannot_create_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let invalid_name = create_client(
        &db,
        &admin,
        CreateClient {
            name: "".to_string(),
            phone: Some("+359123456".to_string()),
            email: Some("email@example.com".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_name, CreateClientError::Validation(_)));

    let invalid_phone = create_client(
        &db,
        &admin,
        CreateClient {
            name: "John Doe".to_string(),
            phone: Some("invalid_phone".to_string()),
            email: Some("email@example.com".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_phone, CreateClientError::Validation(_)));

    let invalid_email = create_client(
        &db,
        &admin,
        CreateClient {
            name: "John Doe".to_string(),
            phone: Some("+359123456".to_string()),
            email: Some("invalid_email".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_email, CreateClientError::Validation(_)));
}

/* -------------------- */
/* Update Clients tests */
/* -------------------- */

#[tokio::test]
async fn admin_can_update_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = seed_client(&db).await;

    let updated_id = update_client(
        &db,
        &admin,
        UpdateClient {
            id: client_id,
            name: Some("John Notdoe".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap();

    let result = get_client(&db, &admin, updated_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "John Notdoe".to_string());
    assert_eq!(result.phone, Some("+359123456".to_string()));
    assert_eq!(result.email, Some("email@example.com".to_string()));
}

#[tokio::test]
async fn employee_cannot_update_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = update_client(
        &db,
        &employee,
        UpdateClient {
            id: client_id,
            name: Some("John Notdoe".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateClientError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_update_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = update_client(
        &db,
        &user,
        UpdateClient {
            id: client_id,
            name: Some("John Notdoe".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateClientError::Forbidden));
}

#[tokio::test]
async fn invalid_client_data_cannot_update_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let client_id = seed_client(&db).await;

    let invalid_name = update_client(
        &db,
        &admin,
        UpdateClient {
            id: client_id,
            name: Some("".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_name, UpdateClientError::Validation(_)));

    let invalid_phone = update_client(
        &db,
        &admin,
        UpdateClient {
            id: client_id,
            name: None,
            phone: Some("invalid_phone".to_string()),
            email: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_phone, UpdateClientError::Validation(_)));

    let invalid_email = update_client(
        &db,
        &admin,
        UpdateClient {
            id: client_id,
            name: None,
            phone: None,
            email: Some("invalid_email".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(invalid_email, UpdateClientError::Validation(_)));
}

#[tokio::test]
async fn updating_nonexistent_client_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = update_client(
        &db,
        &admin,
        UpdateClient {
            id: Uuid::new_v4(),
            name: Some("John Notdoe".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateClientError::NotFound));
}

#[tokio::test]
async fn updating_deleted_client_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = seed_client(&db).await;

    delete_client(&db, &admin, client_id).await.unwrap();

    let result = update_client(
        &db,
        &admin,
        UpdateClient {
            id: client_id,
            name: Some("John Notdoe".to_string()),
            phone: None,
            email: None,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, UpdateClientError::NotFound));
}

/* ------------------- */
/* Delete Client tests */
/* ------------------- */

#[tokio::test]
async fn admin_can_delete_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = seed_client(&db).await;

    delete_client(&db, &admin, client_id).await.unwrap();

    let result = get_client(&db, &admin, client_id).await.unwrap_err();
    assert!(matches!(result, GetClientError::NotFound));
}

#[tokio::test]
async fn employee_cannot_delete_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = delete_client(&db, &employee, client_id).await.unwrap_err();

    assert!(matches!(result, DeleteClientError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_delete_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = delete_client(&db, &user, client_id).await.unwrap_err();

    assert!(matches!(result, DeleteClientError::Forbidden));
}

#[tokio::test]
async fn deleting_nonexistent_client_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = delete_client(&db, &admin, Uuid::new_v4())
        .await
        .unwrap_err();

    assert!(matches!(result, DeleteClientError::NotFound));
}

#[tokio::test]
async fn deleting_already_deleted_client_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = seed_client(&db).await;

    delete_client(&db, &admin, client_id).await.unwrap();

    let result = delete_client(&db, &admin, client_id).await.unwrap_err();

    assert!(matches!(result, DeleteClientError::NotFound));
}

/* ---------------- */
/* Get Client tests */
/* ---------------- */

#[tokio::test]
async fn admin_can_get_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = get_client(&db, &admin, client_id).await.unwrap();
    assert!(result.is_some());

    let result = result.unwrap();
    assert_eq!(result.name, "John Doe".to_string());
    assert_eq!(result.phone, Some("+359123456".to_string()));
    assert_eq!(result.email, Some("email@example.com".to_string()));
}

#[tokio::test]
async fn employee_cannot_get_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = get_client(&db, &employee, client_id).await.unwrap_err();

    assert!(matches!(result, GetClientError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_get_client() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;

    let client_id = seed_client(&db).await;

    let result = get_client(&db, &user, client_id).await.unwrap_err();

    assert!(matches!(result, GetClientError::Forbidden));
}

#[tokio::test]
async fn getting_nonexistent_client_returns_none() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = get_client(&db, &admin, Uuid::new_v4()).await.unwrap_err();

    assert!(matches!(result, GetClientError::NotFound));
}

/* ----------------- */
/* List Clients test */
/* ----------------- */

#[tokio::test]
async fn admin_can_list_clients() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    // Seed multiple clients
    for i in 0..5 {
        create_client(
            &db,
            &admin,
            CreateClient {
                name: format!("Client {}", i),
                phone: Some(format!("+35912345{}", i)),
                email: Some(format!("email_{}@example.com", i)),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::clients::list::list_clients(&db, &admin)
        .await
        .unwrap();

    assert_eq!(result.len(), 5);
}

#[tokio::test]
async fn employee_cannot_list_clients() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let admin = admin_actor(&db).await;

    // Seed multiple clients
    for i in 0..5 {
        create_client(
            &db,
            &admin,
            CreateClient {
                name: format!("Client {}", i),
                phone: Some(format!("+35912345{}", i)),
                email: Some(format!("email_{}@example.com", i)),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::clients::list::list_clients(&db, &employee)
        .await
        .unwrap_err();

    assert!(matches!(result, ListClientsError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_list_clients() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let admin = admin_actor(&db).await;

    // Seed multiple clients
    for i in 0..5 {
        create_client(
            &db,
            &admin,
            CreateClient {
                name: format!("Client {}", i),
                phone: Some(format!("+35912345{}", i)),
                email: Some(format!("email_{}@example.com", i)),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::clients::list::list_clients(&db, &user)
        .await
        .unwrap_err();

    assert!(matches!(result, ListClientsError::Forbidden));
}
