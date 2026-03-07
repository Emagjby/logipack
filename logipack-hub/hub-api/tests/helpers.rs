use core_application::roles::Role;
use core_data::entity::user_roles;
use jsonwebtoken::Header;
use serde_json::json;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use sea_orm::sqlx::types::chrono;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use core_application::actor::ActorContext;
use test_infra::test_db;

use hub_api::app;
use hub_api::config::{AuthMode, Config};
use hub_api::state::AppState;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        dotenvy::from_filename(".env.test").ok();
    })
}

fn normalize_private_key(private_pem: &str) -> Vec<u8> {
    private_pem
        .trim()
        .replace("\\n", "\n")
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .into_bytes()
}

pub async fn seed_auth0_user(db: &DatabaseConnection, auth0_sub: &str) {
    use core_data::entity::{employees, roles, user_roles, users};
    use sea_orm::sqlx::types::chrono;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    users::ActiveModel {
        id: Set(user_id),
        name: Set("Test User".into()),
        email: Set(Some(format!("{}@test.com", user_id))),
        auth0_sub: Set(Some(auth0_sub.to_string())),
        password_hash: Set(Some("x".into())),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    let role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("admin"))
        .one(db)
        .await
        .unwrap()
    {
        Some(r) => r,
        None => {
            let role_id = Uuid::new_v4();
            match (roles::ActiveModel {
                id: Set(role_id),
                name: Set("admin".into()),
            })
            .insert(db)
            .await
            {
                Ok(r) => r,
                Err(_) => roles::Entity::find()
                    .filter(roles::Column::Name.eq("admin"))
                    .one(db)
                    .await
                    .unwrap()
                    .expect("admin role should exist"),
            }
        }
    };

    user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role.id),
    }
    .insert(db)
    .await
    .unwrap();

    let employee_id = Uuid::new_v4();
    employees::ActiveModel {
        id: Set(employee_id),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();
}

pub fn sign_test_jwt(
    kid: &str,
    issuer: &str,
    audience: &str,
    sub: &str,
    private_pem: &str,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": audience,
        "iat": now,
        "nbf": now - 1,
        "exp": now + 3600, // 1 hour expiry
    });

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(kid.to_string());

    let key_bytes = normalize_private_key(private_pem);

    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(&key_bytes).unwrap(),
    )
    .unwrap()
}

pub fn test_config() -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        dev_secret: "test_secret".to_string(),
        auth_mode: AuthMode::DevSecret,
        auth0_issuer: None,
        auth0_audience: None,
        auth0_jwks_url: None,
        auth0_jwks_path: None,
    }
}

pub async fn cleanup_db(db: &DatabaseConnection) {
    let tables = [
        "audit_events",
        "shipment_status_history",
        "shipments",
        "employee_offices",
        "employees",
        "user_roles",
        "users",
        "clients",
        "roles",
        "offices",
        "packages",
        "streams",
    ];

    for t in tables {
        db.execute(Statement::from_string(
            DbBackend::Postgres,
            format!("DELETE FROM {}", t),
        ))
        .await
        .unwrap();
    }
}

fn test_auth0_config() -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        dev_secret: "test_secret".to_string(),
        auth_mode: AuthMode::Auth0,
        auth0_issuer: Some("https://test/".to_string()),
        auth0_audience: Some("logipack".to_string()),
        auth0_jwks_url: None,
        auth0_jwks_path: Some(format!(
            "{}/tests/fixtures/jwks.json",
            env!("CARGO_MANIFEST_DIR")
        )),
    }
}

pub async fn setup_auth0_app() -> axum::Router {
    setup();

    let db = test_db().await;
    cleanup_db(&db).await;

    let state = AppState {
        db,
        auth_mode: AuthMode::Auth0,
    };

    app::router(test_auth0_config(), state)
}

pub async fn setup_auth0_app_with_db() -> (Router, DatabaseConnection) {
    setup();

    let db = test_db().await;
    cleanup_db(&db).await;

    let state = AppState {
        db: db.clone(),
        auth_mode: AuthMode::Auth0,
    };

    (app::router(test_auth0_config(), state), db)
}

pub async fn setup_app_with_employee() -> (Router, ActorContext) {
    use core_data::entity::{employees, roles, users};
    use sea_orm::sqlx::types::chrono;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use uuid::Uuid;

    let db = test_db().await;
    cleanup_db(&db).await;

    let user_id = Uuid::new_v4();
    let email = format!("nobody+{}@test.com", user_id);
    users::ActiveModel {
        id: Set(user_id),
        name: Set("Test User".into()),
        email: Set(Some(email.clone())),
        password_hash: Set(Some("x".into())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("employee"))
        .one(&db)
        .await
        .unwrap()
    {
        Some(r) => r,
        None => {
            let role_id = Uuid::new_v4();
            match (roles::ActiveModel {
                id: Set(role_id),
                name: Set("employee".into()),
            })
            .insert(&db)
            .await
            {
                Ok(r) => r,
                Err(_) => roles::Entity::find()
                    .filter(roles::Column::Name.eq("employee"))
                    .one(&db)
                    .await
                    .unwrap()
                    .expect("employee role should exist"),
            }
        }
    };

    // user_roles link
    user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role.id),
    }
    .insert(&db)
    .await
    .unwrap();

    let employee_model = employees::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(&db)
    .await
    .unwrap();

    let employee = ActorContext {
        user_id,
        sub: email,
        roles: vec![Role::Employee],
        employee_id: Some(employee_model.id),
        allowed_office_ids: vec![],
    };

    let state = AppState {
        db,
        auth_mode: AuthMode::DevSecret,
    };

    let cfg = test_config();
    (app::router(cfg, state), employee)
}

pub async fn setup_app() -> Router {
    use core_data::entity::users;
    use sea_orm::sqlx::types::chrono;
    use sea_orm::{ActiveModelTrait, Set};
    use uuid::Uuid;

    let db = test_db().await;
    cleanup_db(&db).await;

    // Seed a default dev user so routes that extract ActorContext
    // can resolve `x-dev-user-sub` without each test seeding a user.
    let _ = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Test User".into()),
        email: Set(Some("nobody@test.com".to_string())),
        auth0_sub: Set(None),
        password_hash: Set(Some("x".into())),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(&db)
    .await;

    let state = AppState {
        db,
        auth_mode: AuthMode::DevSecret,
    };

    let cfg = test_config();
    app::router(cfg, state)
}

pub async fn setup_app_with_db() -> (Router, DatabaseConnection) {
    let db = test_db().await;

    cleanup_db(&db).await;

    let state = AppState {
        db: db.clone(),
        auth_mode: AuthMode::DevSecret,
    };

    let cfg = test_config();

    let app = app::router(cfg, state);

    (app, db)
}

pub async fn seed_admin_actor(db: &DatabaseConnection) -> ActorContext {
    use core_application::roles::Role;
    use core_data::entity::{roles, user_roles, users};
    use sea_orm::sqlx::types::chrono;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use uuid::Uuid;

    let user_id = Uuid::new_v4();

    // user
    users::ActiveModel {
        id: Set(user_id),
        name: Set("Test User".into()),
        email: Set(Some(format!("admin+{}@test.com", user_id))),
        auth0_sub: Set(None),
        password_hash: Set(Some("x".into())),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    // role row (admin) - reuse if exists
    let role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("admin"))
        .one(db)
        .await
        .unwrap()
    {
        Some(r) => r,
        None => {
            let role_id = Uuid::new_v4();
            match (roles::ActiveModel {
                id: Set(role_id),
                name: Set("admin".into()),
            })
            .insert(db)
            .await
            {
                Ok(r) => r,
                Err(_) => roles::Entity::find()
                    .filter(roles::Column::Name.eq("admin"))
                    .one(db)
                    .await
                    .unwrap()
                    .expect("admin role should exist"),
            }
        }
    };

    // user_roles link
    user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role.id),
    }
    .insert(db)
    .await
    .unwrap();

    let email = format!("admin+{}@test.com", user_id);

    ActorContext {
        user_id,
        sub: email.clone(),
        roles: vec![Role::Admin],
        employee_id: None,
        allowed_office_ids: vec![],
    }
}

pub async fn seed_employee(db: &DatabaseConnection) -> ActorContext {
    use core_application::roles::Role;
    use core_data::entity::{roles, user_roles, users};
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use uuid::Uuid;

    let user_id = Uuid::new_v4();

    // user
    users::ActiveModel {
        id: Set(user_id),
        name: Set("Test User".into()),
        email: Set(Some(format!("employee+{}@test.com", user_id))),
        password_hash: Set(Some("x".into())),
        auth0_sub: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    // role row (employee) - reuse if exists
    let role = match roles::Entity::find()
        .filter(roles::Column::Name.eq("employee"))
        .one(db)
        .await
        .unwrap()
    {
        Some(r) => r,
        None => {
            let role_id = Uuid::new_v4();
            roles::ActiveModel {
                id: Set(role_id),
                name: Set("employee".into()),
            }
            .insert(db)
            .await
            .unwrap()
        }
    };

    // user_roles link
    user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role.id),
    }
    .insert(db)
    .await
    .unwrap();

    let email = format!("employee+{}@test.com", user_id);

    ActorContext {
        user_id,
        sub: email.clone(),
        roles: vec![Role::Employee],
        employee_id: None,
        allowed_office_ids: vec![],
    }
}

pub async fn seed_client(db: &DatabaseConnection) -> Uuid {
    use core_data::entity::clients;
    use sea_orm::{ActiveModelTrait, Set};
    use uuid::Uuid;

    let id = Uuid::new_v4();

    clients::ActiveModel {
        id: Set(id),
        name: Set("Test Client".into()),
        phone: Set(None),
        email: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_office(db: &DatabaseConnection) -> Uuid {
    use core_data::entity::offices;
    use sea_orm::{ActiveModelTrait, Set};
    use uuid::Uuid;

    let id = Uuid::new_v4();

    offices::ActiveModel {
        id: Set(id),
        name: Set("Main Office".into()),
        city: Set("Test City".into()),
        address: Set("Test Address".into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_employee_record(db: &DatabaseConnection) -> Uuid {
    use core_data::entity::employees;
    use sea_orm::{ActiveModelTrait, Set};

    let user_id = seed_user_for_employee(db).await;
    let id = Uuid::new_v4();

    employees::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn seed_user_for_employee(db: &DatabaseConnection) -> Uuid {
    use core_data::entity::users;
    use sea_orm::{ActiveModelTrait, Set};

    let id = Uuid::new_v4();

    users::ActiveModel {
        id: Set(id),
        name: Set("Test User".into()),
        email: Set(Some(format!("user+{}@test.com", id))),
        auth0_sub: Set(None),
        password_hash: Set(Some("x".into())),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(db)
    .await
    .unwrap();

    id
}

pub async fn setup_app_with_admin() -> (Router, DatabaseConnection, ActorContext) {
    let db = test_db().await;

    cleanup_db(&db).await;

    let state = AppState {
        db: db.clone(),
        auth_mode: AuthMode::DevSecret,
    };

    let cfg = test_config();

    let app = app::router(cfg, state);

    let admin_actor = seed_admin_actor(&db).await;

    (app, db, admin_actor)
}
