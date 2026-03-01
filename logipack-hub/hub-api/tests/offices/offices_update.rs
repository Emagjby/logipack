use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use hub_api::dto::offices::UpdateOfficeResponse;

use crate::helpers::{seed_employee, seed_office, setup_app_with_admin, setup_app_with_db};

#[tokio::test]
async fn admin_can_update_office() {
    let (app, db, admin) = setup_app_with_admin().await;
    let office_id = seed_office(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/offices/{}", office_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Office",
                        "city": "Updated City",
                        "address": "Updated Address"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: UpdateOfficeResponse = serde_json::from_slice(&body).unwrap();
    let office: UpdateOfficeResponse = body;

    assert_ne!(office.office.id, String::new());
}

#[tokio::test]
async fn admin_update_office_invalid_uuid() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri("/admin/offices/not-a-uuid")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Office"
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
async fn admin_update_office_not_found() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/offices/{}", Uuid::new_v4()))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Office"
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
async fn admin_update_office_invalid_json() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/offices/{}", Uuid::new_v4()))
                .body(Body::from("{invalid"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn employee_cannot_update_office() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;
    let office_id = seed_office(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/offices/{}", office_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Office"
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
async fn no_role_cannot_update_office() {
    let (app, db) = setup_app_with_db().await;
    let office_id = seed_office(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .header("content-type", "application/json")
                .method(Method::PUT)
                .uri(format!("/admin/offices/{}", office_id))
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Updated Office"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
