use axum::{body::Body, extract::Request, http::StatusCode};
use http_body_util::BodyExt;
use sea_orm::sqlx::types::chrono;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tower::ServiceExt;
use uuid::Uuid;

use core_application::shipments::create::{CreateShipment, create_shipment};
use core_data::entity::{employee_offices, employees, roles, user_roles, users};
use hub_api::dto::shipments::{ShipmentDetail, ShipmentListItem};

#[allow(dead_code)]
mod helpers;
use helpers::{seed_client, seed_office, setup_app_with_admin, setup_app_with_employee};

#[tokio::test]
async fn list_shipments_empty() {
    let (app, employee) = setup_app_with_employee().await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_shipments_returns_rows() {
    let (app, db, admin) = setup_app_with_admin().await;

    let client = seed_client(&db).await;
    let office = seed_office(&db).await;

    create_shipment(
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

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Vec<ShipmentListItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].current_status, "NEW");
}

#[tokio::test]
async fn get_shipment_404() {
    let (app, employee) = setup_app_with_employee().await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/shipments/{}", Uuid::new_v4()))
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_shipment_returns_detail() {
    let (app, db, admin) = setup_app_with_admin().await;

    let client = seed_client(&db).await;
    let office = seed_office(&db).await;

    let shipment = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office),
            notes: Some("hello".into()),
        },
    )
    .await
    .unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/shipments/{}", shipment))
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", admin.sub.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: ShipmentDetail = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.id.to_string(), shipment.to_string());
    assert_eq!(body.current_status, "NEW");
}

#[tokio::test]
async fn employee_list_and_get_only_current_office_shipments() {
    let (app, db, admin) = setup_app_with_admin().await;

    let client = seed_client(&db).await;
    let office1 = seed_office(&db).await;
    let office2 = seed_office(&db).await;

    let shipment_in_scope = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office1),
            notes: None,
        },
    )
    .await
    .unwrap();

    let shipment_out_of_scope = create_shipment(
        &db,
        &admin,
        CreateShipment {
            client_id: client,
            current_office_id: Some(office2),
            notes: None,
        },
    )
    .await
    .unwrap();

    let employee_user_id = Uuid::new_v4();
    let employee_email = format!("employee+{}@test.com", employee_user_id);

    users::ActiveModel {
        id: Set(employee_user_id),
        name: Set("Employee User".to_string()),
        email: Set(Some(employee_email.clone())),
        password_hash: Set(Some("x".to_string())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let employee_role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("employee"))
        .one(&db)
        .await
        .unwrap()
    {
        Some(role) => role,
        None => roles::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("employee".to_string()),
        }
        .insert(&db)
        .await
        .unwrap(),
    };

    user_roles::ActiveModel {
        user_id: Set(employee_user_id),
        role_id: Set(employee_role.id),
    }
    .insert(&db)
    .await
    .unwrap();

    let employee_id = employees::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(employee_user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(&db)
    .await
    .unwrap()
    .id;

    employee_offices::ActiveModel {
        employee_id: Set(employee_id),
        office_id: Set(office1),
    }
    .insert(&db)
    .await
    .unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/shipments")
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee_email.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let items: Vec<ShipmentListItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, shipment_in_scope.to_string());

    let res_forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/shipments/{}", shipment_out_of_scope))
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee_email.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_forbidden.status(), StatusCode::NOT_FOUND);

    let res_allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/shipments/{}", shipment_in_scope))
                .header("x-dev-secret", "test_secret")
                .header("x-dev-user-sub", employee_email)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_allowed.status(), StatusCode::OK);
}
