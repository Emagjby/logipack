use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use http_body_util::BodyExt;
use hub_api::dto::offices::CreateOfficeResponse;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::helpers::{seed_employee, setup_app_with_admin, setup_app_with_db};

#[tokio::test]
async fn admin_can_create_office() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/offices")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Main Office",
                        "city": "Test City",
                        "address": "123 Test Street",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: CreateOfficeResponse = serde_json::from_slice(&body).unwrap();
    let office: CreateOfficeResponse = body;

    assert_ne!(office.office.id, String::new());
}

#[tokio::test]
async fn employee_cannot_create_office() {
    let (app, db) = setup_app_with_db().await;

    let employee = seed_employee(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/offices")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Main Office",
                        "city": "Test City",
                        "address": "123 Test Street",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn no_role_cannot_create_office() {
    let (app, _db) = setup_app_with_db().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/offices")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Main Office",
                        "city": "Test City",
                        "address": "123 Test Street",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_office_invalid_json() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/offices")
                .body(Body::from("{invalid"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
