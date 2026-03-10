use axum::{body::Body, extract::Request, http::StatusCode};
use chrono::{Duration, Utc};
use core_data::entity::{clients, offices, shipments};
use http_body_util::BodyExt;
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

#[allow(dead_code)]
mod helpers;
use helpers::{seed_admin_actor, seed_employee, setup_app_with_db};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
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

async fn insert_shipment(
    db: &DatabaseConnection,
    client_id: Uuid,
    status: &str,
    office_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
) -> Uuid {
    let id = Uuid::new_v4();
    shipments::ActiveModel {
        id: Set(id),
        client_id: Set(client_id),
        current_status: Set(status.to_string()),
        current_office_id: Set(office_id),
        created_at: Set(created_at.into()),
        updated_at: Set(created_at.into()),
    }
    .insert(db)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn reports_are_admin_only() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/reports/shipments-by-status")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn shipments_by_status_and_client_reports_return_tabular_rows() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;
    let admin = seed_admin_actor(&db).await;

    let base = Utc::now() - Duration::days(5);
    let client_a = insert_client(&db, "Acme", base).await;
    let client_b = insert_client(&db, "Beta", base + Duration::hours(2)).await;
    let office = insert_office(&db, "Sofia HQ", base).await;

    insert_shipment(
        &db,
        client_a,
        "NEW",
        Some(office),
        base + Duration::hours(1),
    )
    .await;
    insert_shipment(
        &db,
        client_a,
        "DELIVERED",
        Some(office),
        base + Duration::hours(3),
    )
    .await;
    insert_shipment(
        &db,
        client_b,
        "NEW",
        Some(office),
        base + Duration::hours(4),
    )
    .await;

    let status_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/reports/shipments-by-status")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_res.status(), StatusCode::OK);
    let status_body = status_res.into_body().collect().await.unwrap().to_bytes();
    let status_json: serde_json::Value = serde_json::from_slice(&status_body).unwrap();

    assert_eq!(status_json["report_name"], "shipments-by-status");
    assert_eq!(
        status_json["columns"],
        serde_json::json!(["status", "shipment_count"])
    );
    assert_eq!(
        status_json["rows"],
        serde_json::json!([["DELIVERED", 1], ["NEW", 2]])
    );

    let client_res = app
        .oneshot(
            Request::builder()
                .uri("/reports/shipments-by-client")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(client_res.status(), StatusCode::OK);
    let client_body = client_res.into_body().collect().await.unwrap().to_bytes();
    let client_json: serde_json::Value = serde_json::from_slice(&client_body).unwrap();

    assert_eq!(client_json["report_name"], "shipments-by-client");
    assert_eq!(
        client_json["columns"],
        serde_json::json!(["client_id", "client_name", "shipment_count"])
    );
    assert_eq!(
        client_json["rows"],
        serde_json::json!([
            [client_a.to_string(), "Acme", 2],
            [client_b.to_string(), "Beta", 1]
        ])
    );
}

#[tokio::test]
async fn shipments_by_period_report_honors_bucket_and_ordering() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;
    let admin = seed_admin_actor(&db).await;

    let base = Utc::now() - Duration::days(40);
    let client = insert_client(&db, "Gamma", base).await;
    let office = insert_office(&db, "Varna Port", base).await;

    insert_shipment(&db, client, "NEW", Some(office), base + Duration::days(1)).await;
    insert_shipment(&db, client, "NEW", Some(office), base + Duration::days(9)).await;
    insert_shipment(
        &db,
        client,
        "DELIVERED",
        Some(office),
        base + Duration::days(18),
    )
    .await;

    let uri = format!(
        "/reports/shipments-by-period?from={}&to={}&bucket=week",
        (base.date_naive() + Duration::days(1)).format("%Y-%m-%d"),
        (base.date_naive() + Duration::days(25)).format("%Y-%m-%d"),
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri(uri)
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
    let rows = json["rows"].as_array().unwrap();

    assert_eq!(json["report_name"], "shipments-by-period");
    assert_eq!(
        json["columns"],
        serde_json::json!(["bucket_start", "shipment_count"])
    );
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][1], 1);
    assert_eq!(rows[1][1], 1);
    assert_eq!(rows[2][1], 1);
}

#[tokio::test]
async fn plain_to_date_uses_next_day_exclusive_upper_bound() {
    let _guard = test_lock().lock().await;
    let (app, db) = setup_app_with_db().await;
    let admin = seed_admin_actor(&db).await;

    let day = (Utc::now() - Duration::days(3)).date_naive();
    let from = chrono::DateTime::from_naive_utc_and_offset(day.and_hms_opt(0, 0, 0).unwrap(), Utc);
    let included = chrono::DateTime::from_naive_utc_and_offset(
        day.and_hms_micro_opt(23, 59, 59, 999_500).unwrap(),
        Utc,
    );
    let excluded = chrono::DateTime::from_naive_utc_and_offset(
        (day + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap(),
        Utc,
    );

    let client = insert_client(&db, "Boundary Client", from).await;
    let office = insert_office(&db, "Boundary Office", from).await;

    insert_shipment(&db, client, "NEW", Some(office), included).await;
    insert_shipment(&db, client, "DELIVERED", Some(office), excluded).await;

    let uri = format!(
        "/reports/shipments-by-status?from={}&to={}",
        day.format("%Y-%m-%d"),
        day.format("%Y-%m-%d"),
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri(uri)
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

    assert_eq!(json["rows"], serde_json::json!([["NEW", 1]]));
}
