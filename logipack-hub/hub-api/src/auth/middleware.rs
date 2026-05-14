use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::debug;
use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use std::time::Duration;

use crate::auth::{
    claims::Claims,
    jwks::{cache_is_fresh, cache_keys, get_cached_key, load_jwks_from_path, load_jwks_from_url},
};

#[derive(Debug, Clone)]
pub struct Auth0Config {
    pub issuer: String,
    pub audience: String,

    /// If Some, used for network JWKS fetch
    pub jwks_url: Option<String>,

    /// If Some, use this JSON directly
    pub local_jwks_json: Option<String>,

    /// If Some, load local JWKS from file
    pub local_jwks_path: Option<String>,

    /// Cache TTL for JWKS refresh
    pub jwks_cache_ttl: Duration,
}

fn unauthorized() -> Response {
    StatusCode::UNAUTHORIZED.into_response()
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    since_the_epoch.as_secs() as i64
}

fn validate_claims(cfg: &Auth0Config, claims: &Claims) -> Result<(), ()> {
    // issuer
    if claims.iss != cfg.issuer {
        return Err(());
    }

    // audience
    if !claims.aud.contains(&cfg.audience) {
        return Err(());
    }

    // time
    let now = now_unix();
    if claims.exp < now {
        return Err(());
    }
    if let Some(nbf) = claims.nbf
        && nbf > now
    {
        return Err(());
    }

    Ok(())
}

async fn load_and_cache_jwks(cfg: &Auth0Config) -> Result<(), ()> {
    // prefer local modes first
    if let Some(raw) = &cfg.local_jwks_json {
        let jwks = load_jwks_from_path(raw).map_err(|_| ())?;
        cache_keys(&jwks).map_err(|_| ())?;
        return Ok(());
    }

    if let Some(path) = &cfg.local_jwks_path {
        let jwks = load_jwks_from_path(path).map_err(|_| ())?;
        cache_keys(&jwks).map_err(|_| ())?;
        return Ok(());
    }

    // network
    let url = cfg.jwks_url.as_deref().ok_or(())?;
    let jwks = load_jwks_from_url(url).await.map_err(|_| ())?;
    cache_keys(&jwks).map_err(|_| ())?;
    Ok(())
}

/// Middleware:
/// - Read Authorization: Bearer token
/// - Decode header -> kid
/// - Ensure decoding key exists (cached or fetched)
/// - Verify RS256 signature
/// - Validate iss/aud/exp/nbf
/// - Store Claims in request extensions
pub async fn auth0_jwt_middleware(
    mut req: Request<Body>,
    next: Next,
    cfg: Auth0Config,
) -> Response {
    let authz = match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(v) => v,
        None => {
            debug!("auth missing authorization header");
            return unauthorized();
        }
    };

    let authz = match authz.to_str() {
        Ok(s) => s,
        Err(_) => {
            debug!("auth invalid authorization header encoding");
            return unauthorized();
        }
    };

    let token = match authz
        .strip_prefix("Bearer ")
        .or_else(|| authz.strip_prefix("bearer "))
    {
        Some(t) => t,
        _ => {
            debug!("auth invalid bearer prefix");
            return unauthorized();
        }
    };

    // get kid
    let header = match decode_header(token) {
        Ok(h) => h,
        Err(_) => {
            debug!("auth failed to decode jwt header");
            return unauthorized();
        }
    };

    let kid = match header.kid {
        Some(k) => k,
        None => {
            debug!("auth missing jwt kid");
            return unauthorized();
        }
    };

    // ensure we have a key for kid; refresh JWKS if needed
    let mut key = get_cached_key(&kid);

    if key.is_none() {
        // if cache not fresh, load jwks; if fresh but kid missing, still try reload once
        if load_and_cache_jwks(&cfg).await.is_err() {
            debug!("auth jwks load failed kid={}", kid);
        }
        key = get_cached_key(&kid);
    } else if !cache_is_fresh(cfg.jwks_cache_ttl) {
        // background refresh to keep keys warm
        if load_and_cache_jwks(&cfg).await.is_err() {
            debug!("auth jwks refresh failed kid={}", kid);
        }
    }

    let key = match key {
        Some(k) => k,
        None => {
            debug!("auth jwks missing key kid={}", kid);
            return unauthorized();
        }
    };

    let mut validation = Validation::new(Algorithm::RS256);
    let audience = std::slice::from_ref(&cfg.audience);
    let issuer = std::slice::from_ref(&cfg.issuer);
    validation.set_audience(audience);
    validation.set_issuer(issuer);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let token_data = match decode::<Claims>(token, &key, &validation) {
        Ok(td) => td,
        Err(_) => {
            debug!("auth jwt decode failed");
            return unauthorized();
        }
    };

    // extra explicit claims checks
    if validate_claims(&cfg, &token_data.claims).is_err() {
        debug!(
            "auth claims validation failed iss={} aud={:?} exp={} nbf={:?}",
            token_data.claims.iss,
            token_data.claims.aud,
            token_data.claims.exp,
            token_data.claims.nbf,
        );
        return unauthorized();
    }

    // Stash verfied claims in extensions for later extractor
    req.extensions_mut().insert(token_data.claims);

    next.run(req).await
}
