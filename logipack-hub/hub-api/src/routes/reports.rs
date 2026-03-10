use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use core_data::repository::reporting_repo::{PeriodBucket, ReportFilters, ReportingRepo};
use serde::Deserialize;

use crate::{dto::reports::ReportResponse, error::ApiError, policy, state::AppState};

use core_application::actor::ActorContext;

#[derive(Debug, Deserialize)]
struct ReportQuery {
    from: Option<String>,
    to: Option<String>,
    bucket: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shipments-by-status", get(shipments_by_status_handler))
        .route("/shipments-by-office", get(shipments_by_office_handler))
        .route("/shipments-by-client", get(shipments_by_client_handler))
        .route("/shipments-by-period", get(shipments_by_period_handler))
}

async fn shipments_by_status_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ReportResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let filters = parse_filters(&query)?;
    let report = ReportingRepo::shipments_by_status(&state.db, &filters)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(to_report_response(report)))
}

async fn shipments_by_office_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ReportResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let filters = parse_filters(&query)?;
    let report = ReportingRepo::shipments_by_office(&state.db, &filters)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(to_report_response(report)))
}

async fn shipments_by_client_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ReportResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let filters = parse_filters(&query)?;
    let report = ReportingRepo::shipments_by_client(&state.db, &filters)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(to_report_response(report)))
}

async fn shipments_by_period_handler(
    State(state): State<AppState>,
    actor: ActorContext,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ReportResponse>, ApiError> {
    policy::require_admin(&actor)
        .map_err(|_| ApiError::forbidden("access_denied", "Access denied"))?;

    let filters = parse_filters(&query)?;
    let bucket = parse_bucket(query.bucket.as_deref())?;
    let report = ReportingRepo::shipments_by_period(&state.db, &filters, bucket)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(to_report_response(report)))
}

fn parse_filters(query: &ReportQuery) -> Result<ReportFilters, ApiError> {
    let from = parse_optional_from_datetime(query.from.as_deref())?;
    let parsed_to = parse_optional_to_datetime(query.to.as_deref())?;
    let to = parsed_to.as_ref().map(|bound| bound.exclusive);

    if let (Some(from), Some(to)) = (from.as_ref(), parsed_to.as_ref())
        && from > &to.validation_upper
    {
        return Err(ApiError::bad_request(
            "invalid_period",
            "`from` must be less than or equal to `to`",
        ));
    }

    Ok(ReportFilters { from, to })
}

#[derive(Debug, Clone)]
struct ParsedUpperBound {
    exclusive: DateTime<Utc>,
    validation_upper: DateTime<Utc>,
}

fn parse_optional_from_datetime(raw: Option<&str>) -> Result<Option<DateTime<Utc>>, ApiError> {
    let Some(parsed) = parse_optional_datetime_input(raw)? else {
        return Ok(None);
    };

    Ok(Some(match parsed {
        ParsedDateTimeInput::Instant(value) => value,
        ParsedDateTimeInput::Date(value) => at_midnight(value),
    }))
}

fn parse_optional_to_datetime(raw: Option<&str>) -> Result<Option<ParsedUpperBound>, ApiError> {
    let Some(parsed) = parse_optional_datetime_input(raw)? else {
        return Ok(None);
    };

    Ok(Some(match parsed {
        ParsedDateTimeInput::Instant(value) => ParsedUpperBound {
            exclusive: value,
            validation_upper: value,
        },
        ParsedDateTimeInput::Date(value) => ParsedUpperBound {
            exclusive: at_midnight(value + Duration::days(1)),
            validation_upper: DateTime::from_naive_utc_and_offset(
                value
                    .and_hms_nano_opt(23, 59, 59, 999_999_999)
                    .expect("valid end-of-day time"),
                Utc,
            ),
        },
    }))
}

#[derive(Debug, Clone)]
enum ParsedDateTimeInput {
    Instant(DateTime<Utc>),
    Date(NaiveDate),
}

fn parse_optional_datetime_input(
    raw: Option<&str>,
) -> Result<Option<ParsedDateTimeInput>, ApiError> {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if raw.contains('T') {
        let parsed = DateTime::parse_from_rfc3339(raw).map_err(|_| {
            ApiError::bad_request(
                "invalid_datetime",
                "Datetime filters must be RFC3339 or YYYY-MM-DD",
            )
        })?;
        return Ok(Some(ParsedDateTimeInput::Instant(
            parsed.with_timezone(&Utc),
        )));
    }

    let parsed = NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| {
        ApiError::bad_request(
            "invalid_date",
            "Date filters must use YYYY-MM-DD or RFC3339",
        )
    })?;
    Ok(Some(ParsedDateTimeInput::Date(parsed)))
}

fn at_midnight(date: NaiveDate) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(
        date.and_hms_opt(0, 0, 0).expect("valid day-start time"),
        Utc,
    )
}

fn parse_bucket(raw: Option<&str>) -> Result<PeriodBucket, ApiError> {
    match raw.unwrap_or("day").trim().to_ascii_lowercase().as_str() {
        "day" => Ok(PeriodBucket::Day),
        "week" => Ok(PeriodBucket::Week),
        "month" => Ok(PeriodBucket::Month),
        _ => Err(ApiError::bad_request(
            "invalid_bucket",
            "`bucket` must be one of: day, week, month",
        )),
    }
}

fn to_report_response(
    report: core_data::repository::reporting_repo::TabularReport,
) -> ReportResponse {
    ReportResponse {
        report_name: report.report_name,
        generated_at: Utc::now().to_rfc3339(),
        columns: report.columns,
        rows: report.rows,
    }
}
