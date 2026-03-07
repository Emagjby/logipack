use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Set, Statement,
};
use uuid::Uuid;

use core_data::entity::{employees, roles, user_roles, users};
use core_data::repository::employees_repo::{EmployeeError, EmployeesRepo};
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
async fn create_inserts_row_with_null_deleted_at() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let id = Uuid::new_v4();
    let user_id = seed_user(&db).await;

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();

    let row = employees::Entity::find_by_id(id).one(&db).await.unwrap();
    assert!(row.is_some());
    assert!(row.unwrap().deleted_at.is_none());

    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.name, "Test User".to_string());
}

#[tokio::test]
async fn create_assigns_employee_role() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let role_id = Uuid::new_v4();
    roles::ActiveModel {
        id: Set(role_id),
        name: Set("employee".into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
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
async fn list_returns_only_non_deleted_rows() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id_1 = seed_user(&db).await;
    let user_id_2 = seed_user(&db).await;

    let id_1 = Uuid::new_v4();
    let id_2 = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id_1, user_id_1)
        .await
        .unwrap();
    EmployeesRepo::create_employee(&db, id_2, user_id_2)
        .await
        .unwrap();

    EmployeesRepo::delete_employee(&db, id_1).await.unwrap();

    let rows = EmployeesRepo::list_employees(&db).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].employee.id, id_2);
}

#[tokio::test]
async fn get_returns_record_not_found_for_soft_deleted() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();
    EmployeesRepo::delete_employee(&db, id).await.unwrap();

    let result = EmployeesRepo::get_employee_by_id(&db, id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeError::RecordNotFound));
}

#[tokio::test]
async fn delete_sets_deleted_at_and_reflects_in_get_list() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();

    EmployeesRepo::delete_employee(&db, id).await.unwrap();

    let result = EmployeesRepo::get_employee_by_id(&db, id)
        .await
        .unwrap_err();
    assert!(matches!(result, EmployeeError::RecordNotFound));

    let rows = EmployeesRepo::list_employees(&db).await.unwrap();
    assert!(rows.is_empty());
}

#[tokio::test]
async fn delete_already_deleted_returns_record_not_found() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();
    EmployeesRepo::delete_employee(&db, id).await.unwrap();

    let result = EmployeesRepo::delete_employee(&db, id).await.unwrap_err();
    assert!(matches!(result, EmployeeError::RecordNotFound));
}

#[tokio::test]
async fn create_resurrects_soft_deleted_employee_for_same_user() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();
    EmployeesRepo::delete_employee(&db, id).await.unwrap();

    let new_id = Uuid::new_v4();
    let recreated_id = EmployeesRepo::create_employee(&db, new_id, user_id)
        .await
        .unwrap();

    let employee = employees::Entity::find()
        .filter(employees::Column::UserId.eq(user_id))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert!(employee.deleted_at.is_none());
    assert_eq!(employee.user_id, user_id);
    assert_eq!(recreated_id, employee.id);
}

#[tokio::test]
async fn update_updates_updated_at_does_not_resurrect_deleted() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let user_id = seed_user(&db).await;
    let id = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id, user_id)
        .await
        .unwrap();
    EmployeesRepo::delete_employee(&db, id).await.unwrap();

    let result = EmployeesRepo::update_employee(&db, id).await.unwrap_err();
    assert!(matches!(result, EmployeeError::RecordNotFound));

    let user_id_2 = seed_user(&db).await;
    let id_2 = Uuid::new_v4();

    EmployeesRepo::create_employee(&db, id_2, user_id_2)
        .await
        .unwrap();

    let before = employees::Entity::find_by_id(id_2)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    EmployeesRepo::update_employee(&db, id_2).await.unwrap();

    let after = employees::Entity::find_by_id(id_2)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert!(after.updated_at > before.updated_at);
}
