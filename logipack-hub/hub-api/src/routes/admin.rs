use crate::{
    routes::admin_ep::{audit, clients, employees, offices},
    state::AppState,
};
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/audit", audit::router())
        .nest("/clients", clients::router())
        .nest("/employees", employees::router())
        .nest("/offices", offices::router())
}
