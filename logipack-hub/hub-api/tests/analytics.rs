use axum::{body::Body, extract::Request, http::StatusCode};
use chrono::{Duration, Utc};
use core_application::{actor::ActorContext, roles::Role};
use core_data::entity::{
    clients, employee_offices, employees, offices, roles, shipment_status_history, shipments,
    user_roles, users,
};
use http_body_util::BodyExt;
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

#[allow(dead_code)]
mod helpers;
use helpers::{seed_admin_actor, setup_app_with_db};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

async fn ensure_role(db: &DatabaseConnection, role_name: &str) -> Uuid {
    if let Some(role) = roles::Entity::find()
        .filter(roles::Column::Name.eq(role_name))
        .one(db)
        .await
        .unwrap()
    {
        return role.id;
    }

    let id = Uuid::new_v4();
    roles::ActiveModel {
        id: Set(id),
        name: Set(role_name.to_string()),
    }
    .insert(db)
    .await
    .unwrap()
    .id
}

async fn insert_client(
    db: &DatabaseConnection,
    name: &str,
    created_at: chrono::DateTime<Utc>,
) -> Uuid {
    let id = Uuid::new_v4();
    clients::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        phone: Set(None),
        email: Set(None),
        created_at: Set(created_at.into()),
        updated_at: Set(created_at.into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();
    id
}

async fn insert_office(
    db: &DatabaseConnection,
    name: &str,
    created_at: chrono::DateTime<Utc>,
) -> Uuid {
    let id = Uuid::new_v4();
    offices::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        city: Set("Sofia".to_string()),
        address: Set(format!("{name} address")),
        created_at: Set(created_at.into()),
        updated_at: Set(created_at.into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();
    id
}

async fn insert_employee_record(
    db: &DatabaseConnection,
    name: &str,
    created_at: chrono::DateTime<Utc>,
) -> Uuid {
    let user_id = Uuid::new_v4();
    users::ActiveModel {
        id: Set(user_id),
        name: Set(name.to_string()),
        email: Set(Some(format!("{}@test.com", user_id))),
        password_hash: Set(Some("x".to_string())),
        auth0_sub: Set(None),
        created_at: Set(created_at.into()),
    }
    .insert(db)
    .await
    .unwrap();

    let employee_id = Uuid::new_v4();
    employees::ActiveModel {
        id: Set(employee_id),
        user_id: Set(user_id),
        created_at: Set(created_at.into()),
        updated_at: Set(created_at.into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    employee_id
}

async fn seed_employee_with_office(db: &DatabaseConnection, office_id: Uuid) -> ActorContext {
    let user_id = Uuid::new_v4();
    let email = format!("employee+{}@test.com", user_id);
    users::ActiveModel {
        id: Set(user_id),
        name: Set("Employee Analyst".to_string()),
        email: Set(Some(email.clone())),
        password_hash: Set(Some("x".to_string())),
        auth0_sub: Set(None),
        created_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    let role_id = ensure_role(db, "employee").await;
    user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role_id),
    }
    .insert(db)
    .await
    .unwrap();

    let employee_id = Uuid::new_v4();
    employees::ActiveModel {
        id: Set(employee_id),
        user_id: Set(user_id),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    employee_offices::ActiveModel {
        employee_id: Set(employee_id),
        office_id: Set(office_id),
    }
    .insert(db)
    .await
    .unwrap();

    ActorContext {
        user_id,
        sub: email,
        roles: vec![Role::Employee],
        employee_id: Some(employee_id),
        allowed_office_ids: vec![office_id],
    }
}

async fn insert_shipment(
    db: &DatabaseConnection,
    client_id: Uuid,
    status: &str,
    office_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
) -> Uuid {
    let id = Uuid::new_v4();
    shipments::ActiveModel {
        id: Set(id),
        client_id: Set(client_id),
        current_status: Set(status.to_string()),
        current_office_id: Set(office_id),
        created_at: Set(created_at.into()),
        updated_at: Set(updated_at.into()),
    }
    .insert(db)
    .await
    .unwrap();
    id
}

async fn insert_history(
    db: &DatabaseConnection,
    shipment_id: Uuid,
    from_status: Option<&str>,
    to_status: &str,
    office_id: Option<Uuid>,
    changed_at: chrono::DateTime<Utc>,
) {
    shipment_status_history::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        shipment_id: Set(shipment_id),
        from_status: Set(from_status.map(str::to_string)),
        to_status: Set(to_status.to_string()),
        changed_at: Set(changed_at.into()),
        actor_user_id: Set(None),
        office_id: Set(office_id),
        notes: Set(None),
    }
    .insert(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn admin_overview_returns_totals_deltas_and_series() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;
    let admin = seed_admin_actor(&db).await;

    let now = Utc::now();
    let current = now - Duration::days(2);
    let current_2 = now - Duration::days(1);
    let previous = now - Duration::days(9);

    let client_current = insert_client(&db, "Client Current", current).await;
    let _client_previous = insert_client(&db, "Client Previous", previous).await;
    let office_current = insert_office(&db, "Office Current", current).await;
    let office_previous = insert_office(&db, "Office Previous", previous).await;

    insert_shipment(
        &db,
        client_current,
        "NEW",
        Some(office_current),
        current,
        current,
    )
    .await;
    insert_shipment(
        &db,
        client_current,
        "DELIVERED",
        Some(office_current),
        current_2,
        current_2,
    )
    .await;
    insert_shipment(
        &db,
        client_current,
        "NEW",
        Some(office_previous),
        previous,
        previous,
    )
    .await;

    let assigned_employee_id = insert_employee_record(&db, "Assigned", current).await;
    employee_offices::ActiveModel {
        employee_id: Set(assigned_employee_id),
        office_id: Set(office_current),
    }
    .insert(&db)
    .await
    .unwrap();
    let _unassigned_employee_id = insert_employee_record(&db, "Unassigned", previous).await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/analytics/admin/overview?span=7d")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["total_shipments"], 3);
    assert_eq!(json["shipments_vs_last_period"], 1);
    assert_eq!(json["total_clients"], 2);
    assert_eq!(json["clients_vs_last_period"], 0);
    assert_eq!(json["total_offices"], 2);
    assert_eq!(json["offices_vs_last_period"], 0);
    assert_eq!(json["total_employees"], 2);
    assert_eq!(json["assigned_employees"], 1);
    assert_eq!(json["unassigned_employees"], 1);
    assert_eq!(json["shipments_timeseries"].as_array().unwrap().len(), 7);
    assert_eq!(json["employees_timeseries"].as_array().unwrap().len(), 7);
}

#[tokio::test]
async fn employee_overview_is_scoped_to_current_office() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;

    let now = Utc::now();
    let today = now.date_naive();
    let current_start = chrono::DateTime::from_naive_utc_and_offset(
        (today - Duration::days(6)).and_hms_opt(0, 0, 0).unwrap(),
        Utc,
    );
    let before_current = current_start - Duration::days(1);
    let today_start =
        chrono::DateTime::from_naive_utc_and_offset(today.and_hms_opt(0, 0, 0).unwrap(), Utc);

    let office = insert_office(&db, "Sofia HQ", before_current).await;
    let employee = seed_employee_with_office(&db, office).await;
    let client = insert_client(&db, "Scoped Client", before_current).await;

    let shipment_active = insert_shipment(
        &db,
        client,
        "NEW",
        Some(office),
        now - Duration::days(2),
        now - Duration::days(2),
    )
    .await;
    insert_history(
        &db,
        shipment_active,
        None,
        "NEW",
        Some(office),
        now - Duration::days(2),
    )
    .await;

    let shipment_pending = insert_shipment(
        &db,
        client,
        "PROCESSED",
        Some(office),
        now - Duration::days(3),
        now - Duration::days(3),
    )
    .await;
    insert_history(
        &db,
        shipment_pending,
        None,
        "PROCESSED",
        Some(office),
        now - Duration::days(3),
    )
    .await;

    let shipment_in_transit = insert_shipment(
        &db,
        client,
        "IN_TRANSIT",
        Some(office),
        now - Duration::days(4),
        now - Duration::days(4),
    )
    .await;
    insert_history(
        &db,
        shipment_in_transit,
        None,
        "IN_TRANSIT",
        Some(office),
        now - Duration::days(4),
    )
    .await;

    let shipment_delivered_today = insert_shipment(
        &db,
        client,
        "DELIVERED",
        Some(office),
        now - Duration::days(1),
        today_start + Duration::hours(1),
    )
    .await;
    insert_history(
        &db,
        shipment_delivered_today,
        None,
        "NEW",
        Some(office),
        now - Duration::days(1),
    )
    .await;
    insert_history(
        &db,
        shipment_delivered_today,
        Some("NEW"),
        "DELIVERED",
        Some(office),
        today_start + Duration::hours(1),
    )
    .await;

    let shipment_previous_pending = insert_shipment(
        &db,
        client,
        "DELIVERED",
        Some(office),
        before_current,
        now - Duration::days(1),
    )
    .await;
    insert_history(
        &db,
        shipment_previous_pending,
        None,
        "NEW",
        Some(office),
        before_current,
    )
    .await;
    insert_history(
        &db,
        shipment_previous_pending,
        Some("NEW"),
        "DELIVERED",
        Some(office),
        now - Duration::days(2),
    )
    .await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/analytics/employee/overview?span=7d")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["active_shipments"], 3);
    assert_eq!(json["active_vs_last_period"], 2);
    assert_eq!(json["pending_shipments"], 2);
    assert_eq!(json["pending_vs_last_period"], 1);
    assert_eq!(json["deliveries_today"], 1);
    assert_eq!(json["deliveries_vs_last_period"], 1);
    assert_eq!(json["active_timeseries"].as_array().unwrap().len(), 7);
    assert_eq!(json["pending_timeseries"].as_array().unwrap().len(), 7);
    assert_eq!(
        json["active_timeseries"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["value"],
        3
    );
    assert_eq!(
        json["pending_timeseries"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["value"],
        2
    );
    assert_eq!(
        json["deliveries_timeseries"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["value"],
        1
    );
}
