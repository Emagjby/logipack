use crate::auth::middleware::Auth0Config;
use crate::auth::middleware::auth0_jwt_middleware;
use crate::config::AuthMode;
use crate::config::Config;
use crate::state::AppState;
use axum::{Router, routing::get};

use crate::routes;

fn apply_auth_layer(router: Router<AppState>, cfg: &Config) -> Router<AppState> {
    match cfg.auth_mode {
        AuthMode::DevSecret => {
            let secret = cfg.dev_secret.clone();
            router.layer(axum::middleware::from_fn(move |req, next| {
                crate::dev_secret::dev_secret_middleware(req, next, secret.clone())
            }))
        }

        AuthMode::Auth0 => {
            let auth_cfg = Auth0Config {
                issuer: cfg.auth0_issuer.clone().unwrap(),
                audience: cfg.auth0_audience.clone().unwrap(),
                jwks_url: cfg.auth0_jwks_url.clone(),
                local_jwks_path: cfg.auth0_jwks_path.clone(),
                local_jwks_json: None,
                jwks_cache_ttl: std::time::Duration::from_secs(600),
            };

            router.layer(axum::middleware::from_fn(move |req, next| {
                auth0_jwt_middleware(req, next, auth_cfg.clone())
            }))
        }
    }
}

pub fn router(cfg: Config, state: AppState) -> Router {
    let public_router = Router::new().route("/health", get(routes::health::get_health));

    let protected_router = Router::new()
        .merge(routes::ensure_user::router())
        .merge(routes::me::router())
        .nest("/analytics", routes::analytics::router())
        .nest("/reports", routes::reports::router())
        .nest("/shipments", routes::shipments::router())
        .nest("/admin", routes::admin::router());
    let protected_router = apply_auth_layer(protected_router, &cfg);

    public_router.merge(protected_router).with_state(state)
}
