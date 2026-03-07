use chrono::{Duration, FixedOffset, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde_json::json;
use uuid::Uuid;

use core_data::repository::audit_repo::{AuditCursor, AuditRepo, InsertAuditEvent};
use test_infra::test_db;

async fn cleanup_core_data(db: &DatabaseConnection) {
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

    for table in tables {
        db.execute(Statement::from_string(
            DbBackend::Postgres,
            format!("DELETE FROM {}", table),
        ))
        .await
        .unwrap();
    }
}

fn make_event(index: i64, occurred_at: chrono::DateTime<FixedOffset>) -> InsertAuditEvent {
    InsertAuditEvent {
        id: Uuid::new_v4(),
        occurred_at,
        actor_user_id: None,
        actor_display_name: Some(format!("Actor {index}")),
        action_key: format!("shipment.created.{index}"),
        entity_type: Some("shipment".into()),
        entity_id: Some(format!("shipment-{index}")),
        entity_label: Some(format!("Shipment {index}")),
        office_id: None,
        office_label: None,
        target_route: Some(format!("/app/admin/shipments/shipment-{index}")),
        metadata_json: Some(json!({ "index": index })),
        request_id: Some(format!("req-{index}")),
    }
}

#[test]
fn audit_cursor_roundtrip() {
    let cursor = AuditCursor {
        occurred_at: Utc::now().fixed_offset(),
        id: Uuid::new_v4(),
    };

    let encoded = cursor.encode();
    let decoded = AuditCursor::decode(&encoded).unwrap();

    assert_eq!(decoded, cursor);
}

#[test]
fn audit_cursor_rejects_invalid_values() {
    assert!(AuditCursor::decode("bad-cursor").is_err());
    assert!(AuditCursor::decode("2026-03-07T10:00:00Z|not-a-uuid").is_err());
}

#[tokio::test]
async fn insert_event_persists_row() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let event = make_event(1, Utc::now().fixed_offset());
    let inserted = AuditRepo::insert_event(&db, event.clone()).await.unwrap();

    assert_eq!(inserted.id, event.id);
    assert_eq!(inserted.action_key, event.action_key);
    assert_eq!(inserted.metadata_json, event.metadata_json);
}

#[tokio::test]
async fn list_paginated_returns_stable_descending_pages() {
    let db = test_db().await;
    cleanup_core_data(&db).await;

    let base = Utc::now().fixed_offset();
    for index in 0..3 {
        AuditRepo::insert_event(&db, make_event(index, base - Duration::minutes(index)))
            .await
            .unwrap();
    }

    let first = AuditRepo::list_paginated(&db, 2, None).await.unwrap();
    assert_eq!(first.rows.len(), 2);
    assert!(first.has_next);
    assert!(first.next_cursor.is_some());
    assert!(first.rows[0].occurred_at >= first.rows[1].occurred_at);

    let second = AuditRepo::list_paginated(&db, 2, first.next_cursor.as_ref())
        .await
        .unwrap();
    assert_eq!(second.rows.len(), 1);
    assert!(!second.has_next);
    assert!(second.next_cursor.is_none());
    assert_ne!(first.rows[0].id, second.rows[0].id);
    assert_ne!(first.rows[1].id, second.rows[0].id);
}
