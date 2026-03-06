use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use core_application::shipments::{
    change_status::{ChangeStatus, change_status},
    create::{CreateShipment, create_shipment},
};
use core_domain::shipment::ShipmentStatus;
use tower::ServiceExt;

use crate::helpers::{seed_client, seed_office, setup_app_with_admin};

#[allow(dead_code)]
mod helpers;

#[tokio::test]
async fn read_timeline_returns_ordered_events() {
    let (app, db, admin) = setup_app_with_admin().await;

    let office = seed_office(&db).await;
    let client = seed_client(&db).await;

    let shipment_id = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Accepted,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    change_status(
        &db,
        &admin,
        ChangeStatus {
            shipment_id,
            to_status: ShipmentStatus::Processed,
            to_office_id: Some(office),
            notes: None,
        },
    )
    .await
    .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri(format!("/shipments/{}/timeline", shipment_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let timeline = json.as_array().unwrap();

    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0]["seq"], 2);
    assert_eq!(timeline[1]["seq"], 3);
    assert_eq!(timeline[2]["seq"], 4);

    assert_eq!(timeline[0]["event_type"], "ShipmentCreated");
    assert_eq!(timeline[1]["event_type"], "StatusChanged");
    assert_eq!(timeline[2]["event_type"], "StatusChanged");

    for item in timeline {
        let scb = item.get("scb");
        assert!(scb.is_some() && !scb.unwrap().is_null());
    }
}
