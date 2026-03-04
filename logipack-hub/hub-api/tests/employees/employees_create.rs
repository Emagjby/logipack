use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use http_body_util::BodyExt;
use hub_api::dto::employees::CreateEmployeeResponse;
use sea_orm::EntityTrait;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::helpers::{
    seed_employee, seed_user_for_employee, setup_app_with_admin, setup_app_with_db,
};

#[tokio::test]
async fn admin_can_create_employee() {
    let (app, db, admin) = setup_app_with_admin().await;
    let user_id = seed_user_for_employee(&db).await;
    let user = core_data::entity::users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/employees")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "email": user.email.unwrap(),
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: CreateEmployeeResponse = serde_json::from_slice(&body).unwrap();
    let employee_id = Uuid::parse_str(&body.employee.id).unwrap();

    assert_ne!(employee_id, Uuid::nil());
}

#[tokio::test]
async fn employee_cannot_create_employee() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;
    let user_id = seed_user_for_employee(&db).await;
    let user = core_data::entity::users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/employees")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "email": user.email.unwrap(),
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
async fn no_role_cannot_create_employee() {
    let (app, db) = setup_app_with_db().await;
    let user_id = seed_user_for_employee(&db).await;
    let user = core_data::entity::users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/employees")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "email": user.email.unwrap(),
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
async fn create_employee_invalid_json() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .header("content-type", "application/json")
                .method(Method::POST)
                .uri("/admin/employees")
                .body(Body::from("{invalid"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
