use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use core_application::actor::ActorContext;
use core_data::repository::{
    analytics_repo::{AdminOverviewMetrics, AnalyticsRepo, AnalyticsSpan, EmployeeOverviewMetrics},
    employee_offices_repo::EmployeeOfficesRepo,
};
use serde::Deserialize;

use crate::{
    dto::analytics::{AdminOverviewResponse, EmployeeOverviewResponse, TimeseriesPointDto},
    error::ApiError,
    policy,
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    span: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/overview", get(admin_overview_handler))
        .route("/employee/overview", get(employee_overview_handler))
}

async fn admin_overview_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AdminOverviewResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let span = parse_span(query.span.as_deref())?;
    let metrics = AnalyticsRepo::admin_overview(&state.db, span)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(to_admin_response(metrics)))
}

async fn employee_overview_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<EmployeeOverviewResponse>, ApiError> {
    policy::require_employee(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let span = parse_span(query.span.as_deref())?;
    let metrics = if let Some(employee_id) = actor.employee_id {
        if let Some(office_id) = EmployeeOfficesRepo::current_office_id(&state.db, employee_id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
        {
            AnalyticsRepo::employee_overview(&state.db, office_id, span)
                .await
                .map_err(ApiError::from)?
        } else {
            AnalyticsRepo::empty_employee_overview(span)
        }
    } else {
        AnalyticsRepo::empty_employee_overview(span)
    };

    Ok(Json(to_employee_response(metrics)))
}

fn parse_span(raw: Option<&str>) -> Result<AnalyticsSpan, ApiError> {
    match raw.unwrap_or("30d").trim().to_ascii_lowercase().as_str() {
        "7d" => Ok(AnalyticsSpan::Days7),
        "30d" => Ok(AnalyticsSpan::Days30),
        "90d" => Ok(AnalyticsSpan::Days90),
        _ => Err(ApiError::bad_request(
            "invalid_span",
            "`span` must be one of: 7d, 30d, 90d",
        )),
    }
}

fn to_timeseries(
    points: Vec<core_data::repository::analytics_repo::TimeseriesPoint>,
) -> Vec<TimeseriesPointDto> {
    points
        .into_iter()
        .map(|point| TimeseriesPointDto {
            bucket_start: point.bucket_start,
            value: point.value,
        })
        .collect()
}

fn to_admin_response(metrics: AdminOverviewMetrics) -> AdminOverviewResponse {
    AdminOverviewResponse {
        total_shipments: metrics.total_shipments,
        shipments_vs_last_period: metrics.shipments_vs_last_period,
        shipments_timeseries: to_timeseries(metrics.shipments_timeseries),
        total_clients: metrics.total_clients,
        clients_vs_last_period: metrics.clients_vs_last_period,
        clients_timeseries: to_timeseries(metrics.clients_timeseries),
        total_offices: metrics.total_offices,
        offices_vs_last_period: metrics.offices_vs_last_period,
        offices_timeseries: to_timeseries(metrics.offices_timeseries),
        total_employees: metrics.total_employees,
        assigned_employees: metrics.assigned_employees,
        unassigned_employees: metrics.unassigned_employees,
        employees_timeseries: to_timeseries(metrics.employees_timeseries),
    }
}

fn to_employee_response(metrics: EmployeeOverviewMetrics) -> EmployeeOverviewResponse {
    EmployeeOverviewResponse {
        active_shipments: metrics.active_shipments,
        active_vs_last_period: metrics.active_vs_last_period,
        active_timeseries: to_timeseries(metrics.active_timeseries),
        pending_shipments: metrics.pending_shipments,
        pending_vs_last_period: metrics.pending_vs_last_period,
        pending_timeseries: to_timeseries(metrics.pending_timeseries),
        deliveries_today: metrics.deliveries_today,
        deliveries_vs_last_period: metrics.deliveries_vs_last_period,
        deliveries_timeseries: to_timeseries(metrics.deliveries_timeseries),
    }
}
