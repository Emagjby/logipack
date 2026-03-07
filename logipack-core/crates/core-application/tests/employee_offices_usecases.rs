use core_application::actor::ActorContext;
use core_application::employee_offices::assign::{AssignOffice, AssignOfficeError, assign_office};
use core_application::employee_offices::list::{ListEmployeeOfficesError, list_employee_offices};
use core_application::employee_offices::remove::{RemoveOffice, RemoveOfficeError, remove_office};
use core_application::roles::Role;
use core_data::entity::{employees, offices, users};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, IntoActiveModel,
    Set, Statement,
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

async fn seed_employee_record(db: &DatabaseConnection) -> Uuid {
    let user_id = seed_user(db, Some("employee_rec".to_string())).await;
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

async fn seed_office_record(db: &DatabaseConnection) -> Uuid {
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
    let user = users::Entity::find_by_id(user_id)
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let mut user_model = user.into_active_model();
    user_model.name = Set("Employee Actor".into());
    user_model.update(db).await.unwrap();
    let employee_id = Uuid::new_v4();

    employees::ActiveModel {
        id: Set(employee_id),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

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

/* ----------------------------- */
/* Assign Office to Employee     */
/* ----------------------------- */

#[tokio::test]
async fn admin_can_assign_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap();

    let offices = list_employee_offices(&db, &admin, employee_id)
        .await
        .unwrap();
    assert_eq!(offices.len(), 1);
    assert_eq!(offices[0], office_id);
}

#[tokio::test]
async fn employee_cannot_assign_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = assign_office(
        &db,
        &employee,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, AssignOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_assign_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = assign_office(
        &db,
        &user,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, AssignOfficeError::Forbidden));
}

#[tokio::test]
async fn assign_to_nonexistent_employee_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id: Uuid::new_v4(),
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, AssignOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn assign_nonexistent_office_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;

    let result = assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id: Uuid::new_v4(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, AssignOfficeError::OfficeNotFound));
}

#[tokio::test]
async fn assign_duplicate_returns_already_assigned() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap();

    // Second assign should return AlreadyAssigned
    let result = assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, AssignOfficeError::AlreadyAssigned));

    let offices = list_employee_offices(&db, &admin, employee_id)
        .await
        .unwrap();
    assert_eq!(offices.len(), 1);
}

/* ---------------------------------- */
/* Remove Office from Employee        */
/* ---------------------------------- */

#[tokio::test]
async fn admin_can_remove_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap();

    remove_office(
        &db,
        &admin,
        RemoveOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap();

    let offices = list_employee_offices(&db, &admin, employee_id)
        .await
        .unwrap();
    assert!(offices.is_empty());
}

#[tokio::test]
async fn employee_cannot_remove_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = remove_office(
        &db,
        &employee,
        RemoveOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, RemoveOfficeError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_remove_office() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = remove_office(
        &db,
        &user,
        RemoveOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, RemoveOfficeError::Forbidden));
}

#[tokio::test]
async fn remove_from_nonexistent_employee_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let office_id = seed_office_record(&db).await;

    let result = remove_office(
        &db,
        &admin,
        RemoveOffice {
            employee_id: Uuid::new_v4(),
            office_id,
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, RemoveOfficeError::EmployeeNotFound));
}

#[tokio::test]
async fn remove_nonexistent_office_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;

    let result = remove_office(
        &db,
        &admin,
        RemoveOffice {
            employee_id,
            office_id: Uuid::new_v4(),
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(result, RemoveOfficeError::OfficeNotFound));
}

#[tokio::test]
async fn remove_missing_assignment_is_noop() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = seed_office_record(&db).await;

    // Remove without prior assign — should not error (idempotent)
    remove_office(
        &db,
        &admin,
        RemoveOffice {
            employee_id,
            office_id,
        },
    )
    .await
    .unwrap();
}

/* ----------------------------------- */
/* List Employee Offices               */
/* ----------------------------------- */

#[tokio::test]
async fn admin_can_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;
    let office_id_1 = seed_office_record(&db).await;
    let office_id_2 = seed_office_record(&db).await;

    assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id: office_id_1,
        },
    )
    .await
    .unwrap();

    assign_office(
        &db,
        &admin,
        AssignOffice {
            employee_id,
            office_id: office_id_2,
        },
    )
    .await
    .unwrap();

    let mut offices = list_employee_offices(&db, &admin, employee_id)
        .await
        .unwrap();
    offices.sort();

    let mut expected = vec![office_id_1, office_id_2];
    expected.sort();

    assert_eq!(offices, expected);
}

#[tokio::test]
async fn employee_cannot_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let employee = employee_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;

    let result = list_employee_offices(&db, &employee, employee_id)
        .await
        .unwrap_err();

    assert!(matches!(result, ListEmployeeOfficesError::Forbidden));
}

#[tokio::test]
async fn no_role_cannot_list_offices() {
    let db = test_db().await;
    cleanup(&db).await;

    let user = no_role_actor(&db).await;
    let employee_id = seed_employee_record(&db).await;

    let result = list_employee_offices(&db, &user, employee_id)
        .await
        .unwrap_err();

    assert!(matches!(result, ListEmployeeOfficesError::Forbidden));
}

#[tokio::test]
async fn list_for_nonexistent_employee_returns_not_found() {
    let db = test_db().await;
    cleanup(&db).await;

    let admin = admin_actor(&db).await;

    let result = list_employee_offices(&db, &admin, Uuid::new_v4())
        .await
        .unwrap_err();

    assert!(matches!(result, ListEmployeeOfficesError::EmployeeNotFound));
}
