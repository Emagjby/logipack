use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use hub_api::dto::clients::UpdateClientResponse;

use crate::helpers::{seed_client, seed_employee, setup_app_with_admin, setup_app_with_db};

#[tokio::test]
async fn admin_can_update_client() {
    let (app, db, admin) = setup_app_with_admin().await;
    let client_id = seed_client(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/clients/{}", client_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Client",
                        "email": "updated@example.com",
                        "phone": "+1555555"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: UpdateClientResponse = serde_json::from_slice(&body).unwrap();
    let client = body.client;

    assert_eq!(client.name, "Updated Client");
}

#[tokio::test]
async fn admin_update_client_invalid_uuid() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri("/admin/clients/not-a-uuid")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Client"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_update_client_not_found() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/clients/{}", Uuid::new_v4()))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Client"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_update_client_invalid_json() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/clients/{}", Uuid::new_v4()))
                .body(Body::from("{invalid"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn employee_cannot_update_client() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;
    let client_id = seed_client(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/clients/{}", client_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Client"
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
async fn no_role_cannot_update_client() {
    let (app, db) = setup_app_with_db().await;
    let client_id = seed_client(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/clients/{}", client_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Client"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
