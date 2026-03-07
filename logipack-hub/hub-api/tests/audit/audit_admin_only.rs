use axum::{body::Body, extract::Request, http::StatusCode};
use tower::ServiceExt;

use crate::helpers::{seed_employee, setup_app_with_db};

#[tokio::test]
async fn employee_cannot_list_admin_audit() {
    let (app, db) = setup_app_with_db().await;
    let employee = seed_employee(&db).await;

    let res = app
        .oneshot(
            Request::builder()
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .uri("/admin/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
