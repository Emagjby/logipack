use core_application::actor::ActorContext;
use core_application::employees::create::{CreateEmployee, CreateEmployeeError, create_employee};
use core_application::employees::delete::{DeleteEmployeeError, delete_employee};
use core_application::employees::get::{GetEmployeeError, get_employee};
use core_application::employees::list::ListEmployeesError;
use core_application::employees::update::{UpdateEmployee, UpdateEmployeeError, update_employee};
use core_application::roles::Role;
use core_data::entity::{employees, users};
use core_data::entity::{roles, user_roles};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    IntoActiveModel, QueryFilter, Set, Statement,
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

async fn seed_employee_record(db: &DatabaseConnection, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();

    let user = users::Entity::find_by_id(user_id)
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let mut user_model = user.into_active_model();
    user_model.name = Set("Test Employee".into());
    user_model.update(db).await.unwrap();

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
    let employee_id = seed_employee_record(db, user_id).await;

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

/* --------------------- */
/* Create Employee tests */
/* --------------------- */

#[tokio::test]
async fn admin_can_create_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let employee_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap();

    let result = get_employee(&db, &admin, employee_id).await.unwrap();
    assert_eq!(result.employee.user_id, user_id);
    assert_eq!(result.user.name, "Test User".to_string());
}

#[tokio::test]
async fn creating_employee_assigns_employee_role() {
    let db = test_db().await;
    cleanup(&db).await;

    let role_id = Uuid::new_v4();
    roles::ActiveModel {
        id: Set(role_id),
        name: Set("employee".into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let admin = admin_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap();

    let link = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .filter(user_roles::Column::RoleId.eq(role_id))
        .one(&db)
        .await
        .unwrap();

    assert!(link.is_some());
}

#[tokio::test]
async fn employee_cannot_create_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let result = create_employee(
        &db,
        &employee,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateEmployeeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_create_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let seeded_user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let result = create_employee(
        &db,
        &user,
        CreateEmployee {
            email: seeded_user.email.unwrap(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, CreateEmployeeError::Forbidden));
}

#[tokio::test]
async fn admin_can_create_employee_and_name_is_preserved() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let employee_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap();

    let result = get_employee(&db, &admin, employee_id).await.unwrap();
    assert_eq!(result.user.name, "Test User".to_string());
}

/* --------------------- */
/* Update Employee tests */
/* --------------------- */

#[tokio::test]
async fn admin_can_update_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let updated_id = update_employee(&db, &admin, UpdateEmployee { id: employee_id })
        .await
        .unwrap();

    let result = get_employee(&db, &admin, updated_id).await.unwrap();
    assert_eq!(result.user.name, "Test Employee".to_string());
}

#[tokio::test]
async fn employee_cannot_update_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = update_employee(&db, &employee, UpdateEmployee { id: employee_id })
        .await
        .unwrap_err();

    assert!(matches!(result, UpdateEmployeeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_update_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = update_employee(&db, &user, UpdateEmployee { id: employee_id })
        .await
        .unwrap_err();

    assert!(matches!(result, UpdateEmployeeError::Forbidden));
}

#[tokio::test]
async fn updating_nonexistent_employee_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = update_employee(&db, &admin, UpdateEmployee { id: Uuid::new_v4() })
        .await
        .unwrap_err();

    assert!(matches!(result, UpdateEmployeeError::NotFound));
}

#[tokio::test]
async fn updating_deleted_employee_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    delete_employee(&db, &admin, employee_id).await.unwrap();

    let result = update_employee(&db, &admin, UpdateEmployee { id: employee_id })
        .await
        .unwrap_err();

    assert!(matches!(result, UpdateEmployeeError::NotFound));
}

/* --------------------- */
/* Delete Employee tests */
/* --------------------- */

#[tokio::test]
async fn admin_can_delete_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    delete_employee(&db, &admin, employee_id).await.unwrap();

    let result = get_employee(&db, &admin, employee_id).await.unwrap_err();
    assert!(matches!(result, GetEmployeeError::NotFound));
}

#[tokio::test]
async fn admin_can_recreate_soft_deleted_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let employee_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.clone().unwrap(),
        },
    )
    .await
    .unwrap();

    delete_employee(&db, &admin, employee_id).await.unwrap();

    let recreated_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap();

    let detail = get_employee(&db, &admin, recreated_id).await.unwrap();
    assert!(detail.employee.deleted_at.is_none());
}

#[tokio::test]
async fn admin_create_existing_employee_returns_existing_id() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let employee_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.clone().unwrap(),
        },
    )
    .await
    .unwrap();

    let second_id = create_employee(
        &db,
        &admin,
        CreateEmployee {
            email: user.email.unwrap(),
        },
    )
    .await
    .unwrap();

    assert_eq!(second_id, employee_id);
}

#[tokio::test]
async fn employee_cannot_delete_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = delete_employee(&db, &employee, employee_id)
        .await
        .unwrap_err();

    assert!(matches!(result, DeleteEmployeeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_delete_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = delete_employee(&db, &user, employee_id).await.unwrap_err();

    assert!(matches!(result, DeleteEmployeeError::Forbidden));
}

#[tokio::test]
async fn deleting_nonexistent_employee_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = delete_employee(&db, &admin, Uuid::new_v4())
        .await
        .unwrap_err();

    assert!(matches!(result, DeleteEmployeeError::NotFound));
}

#[tokio::test]
async fn deleting_already_deleted_employee_returns_error() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    delete_employee(&db, &admin, employee_id).await.unwrap();

    let result = delete_employee(&db, &admin, employee_id).await.unwrap_err();

    assert!(matches!(result, DeleteEmployeeError::NotFound));
}

/* ------------------ */
/* Get Employee tests */
/* ------------------ */

#[tokio::test]
async fn admin_can_get_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = get_employee(&db, &admin, employee_id).await.unwrap();
    assert_eq!(result.user.name, "Test Employee".to_string());
    assert_eq!(result.employee.user_id, user_id);
}

#[tokio::test]
async fn employee_cannot_get_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = get_employee(&db, &employee, employee_id).await.unwrap_err();

    assert!(matches!(result, GetEmployeeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_get_employee() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let user_id = seed_user(&db, Some("user".to_string())).await;
    let employee_id = seed_employee_record(&db, user_id).await;

    let result = get_employee(&db, &user, employee_id).await.unwrap_err();

    assert!(matches!(result, GetEmployeeError::Forbidden));
}

#[tokio::test]
async fn getting_nonexistent_employee_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = get_employee(&db, &admin, Uuid::new_v4()).await.unwrap_err();

    assert!(matches!(result, GetEmployeeError::NotFound));
}

/* ------------------- */
/* List Employees test */
/* ------------------- */

#[tokio::test]
async fn admin_can_list_employees() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    for i in 0..5 {
        let user_id = seed_user(&db, Some(format!("user_{i}"))).await;
        let user = users::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        create_employee(
            &db,
            &admin,
            CreateEmployee {
                email: user.email.unwrap(),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::employees::list::list_employees(&db, &admin)
        .await
        .unwrap();

    assert_eq!(result.len(), 5);
}

#[tokio::test]
async fn employee_cannot_list_employees() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let admin = admin_actor(&db).await;

    for i in 0..5 {
        let user_id = seed_user(&db, Some(format!("user_{i}"))).await;
        let user = users::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        create_employee(
            &db,
            &admin,
            CreateEmployee {
                email: user.email.unwrap(),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::employees::list::list_employees(&db, &employee)
        .await
        .unwrap_err();

    assert!(matches!(result, ListEmployeesError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_list_employees() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let admin = admin_actor(&db).await;

    for i in 0..5 {
        let user_id = seed_user(&db, Some(format!("user_{i}"))).await;
        let user = users::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        create_employee(
            &db,
            &admin,
            CreateEmployee {
                email: user.email.unwrap(),
            },
        )
        .await
        .unwrap();
    }

    let result = core_application::employees::list::list_employees(&db, &user)
        .await
        .unwrap_err();

    assert!(matches!(result, ListEmployeesError::Forbidden));
}
