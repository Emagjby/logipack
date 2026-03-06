use std::{env, time::SystemTime};

use axum::{body::Body, extract::Request};
use jsonwebtoken::Header;
use serde_json::json;
use tower::ServiceExt;

use crate::helpers::{
    seed_auth0_user, seed_office, setup_auth0_app, setup_auth0_app_with_db, sign_test_jwt,
};
use core_data::entity::employee_offices;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[allow(dead_code)]
pub mod helpers;

#[tokio::test]
async fn auth0_valid_token_allows_request() {
    let (app, db) = setup_auth0_app_with_db().await;

    let sub = "user|123";
    seed_auth0_user(&db, sub).await;

    let private = env::var("TEST_AUTH0_PRIVATE_PEM").unwrap();

    let token = sign_test_jwt(
        "vPGrStQtI1pBCs8y+UqMe7vR/S90cOiQQJy3BKyEnJI=",
        "https://test/",
        "logipack",
        sub,
        &private,
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn me_returns_employee_office_context() {
    let (app, db) = setup_auth0_app_with_db().await;

    let sub = "user|me-office";
    seed_auth0_user(&db, sub).await;

    let office_id = seed_office(&db).await;
    let user = core_data::entity::users::Entity::find()
        .filter(core_data::entity::users::Column::Auth0Sub.eq(Some(sub.to_string())))
        .one(&db)
        .await
        .unwrap()
        .expect("user for sub should exist");

    let employee = core_data::entity::employees::Entity::find()
        .filter(core_data::entity::employees::Column::UserId.eq(user.id))
        .one(&db)
        .await
        .unwrap()
        .expect("employee for sub should exist");

    employee_offices::ActiveModel {
        employee_id: Set(employee.id),
        office_id: Set(office_id),
    }
    .insert(&db)
    .await
    .unwrap();

    let private = env::var("TEST_AUTH0_PRIVATE_PEM").unwrap();
    let token = sign_test_jwt(
        "vPGrStQtI1pBCs8y+UqMe7vR/S90cOiQQJy3BKyEnJI=",
        "https://test/",
        "logipack",
        sub,
        &private,
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri("/me")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["role"], "admin");
    assert!(json["office_ids"].as_array().is_some());
    assert_eq!(json["current_office_id"], office_id.to_string());
}

#[tokio::test]
async fn auth0_missing_token_is_401() {
    let app = setup_auth0_app().await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth0_wrong_audience_is_401() {
    let app = setup_auth0_app().await;

    let private = env::var("TEST_AUTH0_PRIVATE_PEM").unwrap();

    let token = sign_test_jwt(
        "vPGrStQtI1pBCs8y+UqMe7vR/S90cOiQQJy3BKyEnJI=",
        "https://test/",
        "wrong_audience",
        "user|123",
        &private,
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth0_expired_token_is_401() {
    let app = setup_auth0_app().await;

    let private = env::var("TEST_AUTH0_PRIVATE_PEM").unwrap();

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = json!({
        "iss": "https://test/",
        "aud": "logipack",
        "sub": "user|123",
        "iat": now,
        "nbf": now - 1,
        "exp": now - 10,
    });

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("vPGrStQtI1pBCs8y+UqMe7vR/S90cOiQQJy3BKyEnJI=".to_string());

    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(
            private
                .trim()
                .replace("\\n", "\n")
                .replace("\r\n", "\n")
                .replace("\r", "\n")
                .as_bytes(),
        )
        .unwrap(),
    )
    .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth0_unknown_kid_is_401() {
    let app = setup_auth0_app().await;

    let private = env::var("TEST_AUTH0_PRIVATE_PEM").unwrap();

    let token = sign_test_jwt(
        "unknown_kid",
        "https://test/",
        "logipack",
        "user|123",
        &private,
    );

    let res = app
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}
