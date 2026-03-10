use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use serde_json::{Number, Value as JsonValue};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportingQueryError {
    #[error("db error: {0}")]
    Db(#[from] DbErr),
}

#[derive(Debug, Clone, Default)]
pub struct ReportFilters {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodBucket {
    Day,
    Week,
    Month,
}

impl PeriodBucket {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabularReport {
    pub report_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<JsonValue>>,
}

pub struct ReportingRepo;

const SHIPMENTS_BY_STATUS_SQL: &str = r#"
SELECT
    s.current_status AS status,
    COUNT(*)::bigint AS shipment_count
FROM shipments s
WHERE ($1::timestamptz IS NULL OR s.created_at >= $1::timestamptz)
  AND ($2::timestamptz IS NULL OR s.created_at < $2::timestamptz)
GROUP BY s.current_status
ORDER BY s.current_status ASC
"#;

const SHIPMENTS_BY_OFFICE_SQL: &str = r#"
SELECT
    s.current_office_id::text AS office_id,
    COALESCE(o.name, 'Unassigned') AS office_name,
    COUNT(*)::bigint AS shipment_count
FROM shipments s
LEFT JOIN offices o
    ON o.id = s.current_office_id
WHERE ($1::timestamptz IS NULL OR s.created_at >= $1::timestamptz)
  AND ($2::timestamptz IS NULL OR s.created_at < $2::timestamptz)
GROUP BY s.current_office_id, COALESCE(o.name, 'Unassigned')
ORDER BY office_name ASC, office_id ASC NULLS LAST
"#;

const SHIPMENTS_BY_CLIENT_SQL: &str = r#"
SELECT
    s.client_id::text AS client_id,
    c.name AS client_name,
    COUNT(*)::bigint AS shipment_count
FROM shipments s
JOIN clients c
    ON c.id = s.client_id
WHERE ($1::timestamptz IS NULL OR s.created_at >= $1::timestamptz)
  AND ($2::timestamptz IS NULL OR s.created_at < $2::timestamptz)
GROUP BY s.client_id, c.name
ORDER BY c.name ASC, client_id ASC
"#;

const SHIPMENTS_BY_PERIOD_SQL: &str = r#"
SELECT
    TO_CHAR(
        DATE_TRUNC($3, timezone('UTC', s.created_at)),
        'YYYY-MM-DD"T"HH24:MI:SS"Z"'
    ) AS bucket_start,
    COUNT(*)::bigint AS shipment_count
FROM shipments s
WHERE ($1::timestamptz IS NULL OR s.created_at >= $1::timestamptz)
  AND ($2::timestamptz IS NULL OR s.created_at < $2::timestamptz)
GROUP BY DATE_TRUNC($3, timezone('UTC', s.created_at))
ORDER BY DATE_TRUNC($3, timezone('UTC', s.created_at)) ASC
"#;

impl ReportingRepo {
    pub async fn shipments_by_status(
        db: &DatabaseConnection,
        filters: &ReportFilters,
    ) -> Result<TabularReport, ReportingQueryError> {
        let statement = report_statement(SHIPMENTS_BY_STATUS_SQL, filters);
        let rows = db.query_all(statement).await?;
        let rows = rows
            .into_iter()
            .map(|row| {
                Ok(vec![
                    text(row.try_get("", "status")?),
                    number(row.try_get("", "shipment_count")?),
                ])
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(TabularReport {
            report_name: "shipments-by-status".to_string(),
            columns: vec!["status".to_string(), "shipment_count".to_string()],
            rows,
        })
    }

    pub async fn shipments_by_office(
        db: &DatabaseConnection,
        filters: &ReportFilters,
    ) -> Result<TabularReport, ReportingQueryError> {
        let statement = report_statement(SHIPMENTS_BY_OFFICE_SQL, filters);
        let rows = db.query_all(statement).await?;
        let rows = rows
            .into_iter()
            .map(|row| {
                Ok(vec![
                    nullable_text(row.try_get("", "office_id")?),
                    text(row.try_get("", "office_name")?),
                    number(row.try_get("", "shipment_count")?),
                ])
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(TabularReport {
            report_name: "shipments-by-office".to_string(),
            columns: vec![
                "office_id".to_string(),
                "office_name".to_string(),
                "shipment_count".to_string(),
            ],
            rows,
        })
    }

    pub async fn shipments_by_client(
        db: &DatabaseConnection,
        filters: &ReportFilters,
    ) -> Result<TabularReport, ReportingQueryError> {
        let statement = report_statement(SHIPMENTS_BY_CLIENT_SQL, filters);
        let rows = db.query_all(statement).await?;
        let rows = rows
            .into_iter()
            .map(|row| {
                Ok(vec![
                    text(row.try_get("", "client_id")?),
                    text(row.try_get("", "client_name")?),
                    number(row.try_get("", "shipment_count")?),
                ])
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(TabularReport {
            report_name: "shipments-by-client".to_string(),
            columns: vec![
                "client_id".to_string(),
                "client_name".to_string(),
                "shipment_count".to_string(),
            ],
            rows,
        })
    }

    pub async fn shipments_by_period(
        db: &DatabaseConnection,
        filters: &ReportFilters,
        bucket: PeriodBucket,
    ) -> Result<TabularReport, ReportingQueryError> {
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            SHIPMENTS_BY_PERIOD_SQL,
            vec![
                filters.from.map(|value| value.to_rfc3339()).into(),
                filters.to.map(|value| value.to_rfc3339()).into(),
                bucket.as_sql().to_string().into(),
            ],
        );
        let rows = db.query_all(statement).await?;
        let rows = rows
            .into_iter()
            .map(|row| {
                Ok(vec![
                    text(row.try_get("", "bucket_start")?),
                    number(row.try_get("", "shipment_count")?),
                ])
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(TabularReport {
            report_name: "shipments-by-period".to_string(),
            columns: vec!["bucket_start".to_string(), "shipment_count".to_string()],
            rows,
        })
    }
}

fn report_statement(sql: &str, filters: &ReportFilters) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        sql,
        vec![
            filters.from.map(|value| value.to_rfc3339()).into(),
            filters.to.map(|value| value.to_rfc3339()).into(),
        ],
    )
}

fn text(value: String) -> JsonValue {
    JsonValue::String(value)
}

fn nullable_text(value: Option<String>) -> JsonValue {
    match value {
        Some(value) => JsonValue::String(value),
        None => JsonValue::Null,
    }
}

fn number(value: i64) -> JsonValue {
    JsonValue::Number(Number::from(value))
}
