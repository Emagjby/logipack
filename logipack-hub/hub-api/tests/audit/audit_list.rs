use axum::{body::Body, extract::Request, http::StatusCode};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use core_data::repository::audit_repo::{AuditRepo, InsertAuditEvent};
use hub_api::dto::audit::ListAuditResponse;

use crate::helpers::setup_app_with_admin;

async fn seed_audit_event(
    db: &sea_orm::DatabaseConnection,
    index: i64,
    occurred_at: DateTime<FixedOffset>,
) {
    AuditRepo::insert_event(
        db,
        InsertAuditEvent {
            id: Uuid::new_v4(),
            occurred_at,
            actor_user_id: None,
            actor_display_name: Some(format!("Actor {index}")),
            action_key: if index % 2 == 0 {
                "shipment.created".into()
            } else {
                "office.updated".into()
            },
            entity_type: Some(if index % 2 == 0 { "shipment" } else { "office" }.into()),
            entity_id: Some(format!("entity-{index}")),
            entity_label: Some(format!("Entity {index}")),
            office_id: None,
            office_label: None,
            target_route: Some(format!("/app/admin/entity/{index}")),
            metadata_json: Some(json!({ "index": index })),
            request_id: Some(format!("req-{index}")),
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn list_audit_empty() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri("/admin/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: ListAuditResponse = serde_json::from_slice(&body).unwrap();

    assert!(body.events.is_empty());
    assert_eq!(body.page.limit, 10);
    assert!(!body.page.has_next);
    assert!(body.page.next_cursor.is_none());
}

#[tokio::test]
async fn list_audit_paginates_and_preserves_order() {
    let (app, db, admin) = setup_app_with_admin().await;
    let base = Utc::now().fixed_offset();

    for index in 0..3 {
        seed_audit_event(&db, index, base - Duration::minutes(index)).await;
    }

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri("/admin/audit?limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::OK);

    let first_body = first.into_body().collect().await.unwrap().to_bytes();
    let first_body: ListAuditResponse = serde_json::from_slice(&first_body).unwrap();

    assert_eq!(first_body.events.len(), 2);
    assert!(first_body.page.has_next);
    assert!(first_body.page.next_cursor.is_some());
    assert!(first_body.events[0].occurred_at >= first_body.events[1].occurred_at);
    let encoded_cursor = first_body
        .page
        .next_cursor
        .clone()
        .unwrap()
        .replace("%", "%25")
        .replace("+", "%2B")
        .replace(":", "%3A")
        .replace("|", "%7C");

    let second = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri(format!("/admin/audit?limit=2&cursor={}", encoded_cursor))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::OK);

    let second_body = second.into_body().collect().await.unwrap().to_bytes();
    let second_body: ListAuditResponse = serde_json::from_slice(&second_body).unwrap();

    assert_eq!(second_body.events.len(), 1);
    assert!(!second_body.page.has_next);
    assert!(second_body.page.next_cursor.is_none());
    assert_ne!(first_body.events[0].id, second_body.events[0].id);
    assert_ne!(first_body.events[1].id, second_body.events[0].id);
}
