use axum::{body::Body, extract::Request, http::StatusCode};
use http_body_util::BodyExt;
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

use core_data::entity::employee_offices;
use hub_api::dto::employees::GetEmployeeResponse;

use crate::helpers::{
    seed_employee, seed_employee_record, setup_app_with_admin, setup_app_with_db,
};

#[tokio::test]
async fn admin_can_get_employee() {
    let (app, db, admin) = setup_app_with_admin().await;
    let employee_id = seed_employee_record(&db).await;
    let office_id = crate::helpers::seed_office(&db).await;

    employee_offices::ActiveModel {
        employee_id: Set(employee_id),
        office_id: Set(office_id),
    }
    .insert(&db)
    .await
    .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri(format!("/admin/employees/{}", employee_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: GetEmployeeResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.employee.id, employee_id.to_string());
    assert_eq!(body.employee.offices.as_ref().map(Vec::len), Some(1));
}

#[tokio::test]
async fn admin_get_employee_invalid_uuid() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri("/admin/employees/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_get_employee_not_found() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri(format!("/admin/employees/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn employee_cannot_get_employee() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .uri(format!("/admin/employees/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn no_role_cannot_get_employee() {
    let (app, _db) = setup_app_with_db().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .uri(format!("/admin/employees/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
