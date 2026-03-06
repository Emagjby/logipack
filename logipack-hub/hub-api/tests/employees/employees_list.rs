use axum::{body::Body, extract::Request, http::StatusCode};
use http_body_util::BodyExt;
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

use core_data::entity::employee_offices;
use hub_api::dto::employees::ListEmployeesResponse;

use crate::helpers::{
    seed_employee, seed_employee_record, setup_app_with_admin, setup_app_with_db,
};

#[tokio::test]
async fn list_employees_empty() {
    let (app, _db, admin) = setup_app_with_admin().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .uri("/admin/employees")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: ListEmployeesResponse = serde_json::from_slice(&body).unwrap();

    assert!(body.employees.is_empty());
}

#[tokio::test]
async fn list_employees_returns_rows() {
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
                .uri("/admin/employees")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: ListEmployeesResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.employees.len(), 1);
    assert_eq!(body.employees[0].id, employee_id.to_string());
    assert_eq!(body.employees[0].offices.as_ref().map(Vec::len), Some(1));
}

#[tokio::test]
async fn employee_cannot_list_employees() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .uri("/admin/employees")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn no_role_cannot_list_employees() {
    let (app, _db) = setup_app_with_db().await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", Uuid::new_v4().to_string())
                .uri("/admin/employees")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
