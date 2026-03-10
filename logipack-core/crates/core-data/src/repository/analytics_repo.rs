use chrono::{DateTime, Duration, NaiveDate, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AnalyticsQueryError {
    #[error("db error: {0}")]
    Db(#[from] DbErr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticsSpan {
    Days7,
    Days30,
    Days90,
}

impl AnalyticsSpan {
    pub fn days(self) -> i64 {
        match self {
            Self::Days7 => 7,
            Self::Days30 => 30,
            Self::Days90 => 90,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeseriesPoint {
    pub bucket_start: String,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct AdminOverviewMetrics {
    pub total_shipments: i64,
    pub shipments_vs_last_period: i64,
    pub shipments_timeseries: Vec<TimeseriesPoint>,
    pub total_clients: i64,
    pub clients_vs_last_period: i64,
    pub clients_timeseries: Vec<TimeseriesPoint>,
    pub total_offices: i64,
    pub offices_vs_last_period: i64,
    pub offices_timeseries: Vec<TimeseriesPoint>,
    pub total_employees: i64,
    pub assigned_employees: i64,
    pub unassigned_employees: i64,
    pub employees_timeseries: Vec<TimeseriesPoint>,
}

#[derive(Debug, Clone)]
pub struct EmployeeOverviewMetrics {
    pub active_shipments: i64,
    pub active_vs_last_period: i64,
    pub active_timeseries: Vec<TimeseriesPoint>,
    pub pending_shipments: i64,
    pub pending_vs_last_period: i64,
    pub pending_timeseries: Vec<TimeseriesPoint>,
    pub deliveries_today: i64,
    pub deliveries_vs_last_period: i64,
    pub deliveries_timeseries: Vec<TimeseriesPoint>,
}

pub struct AnalyticsRepo;

const ADMIN_OVERVIEW_SQL: &str = r#"
SELECT
    (SELECT COUNT(*)::bigint FROM shipments) AS total_shipments,
    (SELECT COUNT(*)::bigint FROM shipments WHERE created_at >= $1::timestamptz AND created_at < $2::timestamptz) AS current_shipments,
    (SELECT COUNT(*)::bigint FROM shipments WHERE created_at >= $3::timestamptz AND created_at < $1::timestamptz) AS previous_shipments,
    (SELECT COUNT(*)::bigint FROM clients WHERE deleted_at IS NULL) AS total_clients,
    (SELECT COUNT(*)::bigint FROM clients WHERE deleted_at IS NULL AND created_at >= $1::timestamptz AND created_at < $2::timestamptz) AS current_clients,
    (SELECT COUNT(*)::bigint FROM clients WHERE deleted_at IS NULL AND created_at >= $3::timestamptz AND created_at < $1::timestamptz) AS previous_clients,
    (SELECT COUNT(*)::bigint FROM offices WHERE deleted_at IS NULL) AS total_offices,
    (SELECT COUNT(*)::bigint FROM offices WHERE deleted_at IS NULL AND created_at >= $1::timestamptz AND created_at < $2::timestamptz) AS current_offices,
    (SELECT COUNT(*)::bigint FROM offices WHERE deleted_at IS NULL AND created_at >= $3::timestamptz AND created_at < $1::timestamptz) AS previous_offices,
    (SELECT COUNT(*)::bigint FROM employees WHERE deleted_at IS NULL) AS total_employees,
    (
        SELECT COUNT(DISTINCT e.id)::bigint
        FROM employees e
        JOIN employee_offices eo ON eo.employee_id = e.id
        WHERE e.deleted_at IS NULL
    ) AS assigned_employees
"#;

const DAILY_SHIPMENTS_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
),
counts AS (
    SELECT DATE(timezone('UTC', created_at)) AS bucket_date, COUNT(*)::bigint AS value
    FROM shipments
    WHERE created_at >= $3::timestamptz
      AND created_at < $4::timestamptz
    GROUP BY 1
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(c.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN counts c USING (bucket_date)
ORDER BY b.bucket_date ASC
"#;

const DAILY_CLIENTS_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
),
counts AS (
    SELECT DATE(timezone('UTC', created_at)) AS bucket_date, COUNT(*)::bigint AS value
    FROM clients
    WHERE deleted_at IS NULL
      AND created_at >= $3::timestamptz
      AND created_at < $4::timestamptz
    GROUP BY 1
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(c.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN counts c USING (bucket_date)
ORDER BY b.bucket_date ASC
"#;

const DAILY_OFFICES_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
),
counts AS (
    SELECT DATE(timezone('UTC', created_at)) AS bucket_date, COUNT(*)::bigint AS value
    FROM offices
    WHERE deleted_at IS NULL
      AND created_at >= $3::timestamptz
      AND created_at < $4::timestamptz
    GROUP BY 1
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(c.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN counts c USING (bucket_date)
ORDER BY b.bucket_date ASC
"#;

const DAILY_EMPLOYEES_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
),
counts AS (
    SELECT DATE(timezone('UTC', created_at)) AS bucket_date, COUNT(*)::bigint AS value
    FROM employees
    WHERE deleted_at IS NULL
      AND created_at >= $3::timestamptz
      AND created_at < $4::timestamptz
    GROUP BY 1
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(c.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN counts c USING (bucket_date)
ORDER BY b.bucket_date ASC
"#;

const EMPLOYEE_OVERVIEW_SQL: &str = r#"
WITH latest_before_boundary AS (
    SELECT DISTINCT ON (shipment_id)
        shipment_id,
        to_status,
        office_id
    FROM shipment_status_history
    WHERE changed_at < $2::timestamptz
    ORDER BY shipment_id, changed_at DESC, id DESC
)
SELECT
    (
        SELECT COUNT(*)::bigint
        FROM shipments
        WHERE current_office_id = $1::uuid
          AND current_status NOT IN ('DELIVERED', 'CANCELLED')
    ) AS active_shipments,
    (
        SELECT COUNT(*)::bigint
        FROM latest_before_boundary
        WHERE office_id = $1::uuid
          AND to_status NOT IN ('DELIVERED', 'CANCELLED')
    ) AS previous_active_shipments,
    (
        SELECT COUNT(*)::bigint
        FROM shipments
        WHERE current_office_id = $1::uuid
          AND current_status IN ('NEW', 'ACCEPTED', 'PROCESSED')
    ) AS pending_shipments,
    (
        SELECT COUNT(*)::bigint
        FROM latest_before_boundary
        WHERE office_id = $1::uuid
          AND to_status IN ('NEW', 'ACCEPTED', 'PROCESSED')
    ) AS previous_pending_shipments,
    (
        SELECT COUNT(*)::bigint
        FROM shipment_status_history
        WHERE office_id = $1::uuid
          AND to_status = 'DELIVERED'
          AND changed_at >= $3::timestamptz
          AND changed_at < $4::timestamptz
    ) AS deliveries_today,
    (
        SELECT COUNT(*)::bigint
        FROM shipment_status_history
        WHERE office_id = $1::uuid
          AND to_status = 'DELIVERED'
          AND changed_at >= $5::timestamptz
          AND changed_at < $3::timestamptz
    ) AS deliveries_yesterday
"#;

const ACTIVE_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(snapshot.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN LATERAL (
    SELECT COUNT(*)::bigint AS value
    FROM (
        SELECT DISTINCT ON (ssh.shipment_id)
            ssh.shipment_id,
            ssh.to_status,
            ssh.office_id
        FROM shipment_status_history ssh
        WHERE ssh.changed_at < ((b.bucket_date + INTERVAL '1 day')::timestamp AT TIME ZONE 'UTC')
        ORDER BY ssh.shipment_id, ssh.changed_at DESC, ssh.id DESC
    ) latest
    WHERE latest.office_id = $3::uuid
      AND latest.to_status NOT IN ('DELIVERED', 'CANCELLED')
) snapshot ON TRUE
ORDER BY b.bucket_date ASC
"#;

const PENDING_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(snapshot.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN LATERAL (
    SELECT COUNT(*)::bigint AS value
    FROM (
        SELECT DISTINCT ON (ssh.shipment_id)
            ssh.shipment_id,
            ssh.to_status,
            ssh.office_id
        FROM shipment_status_history ssh
        WHERE ssh.changed_at < ((b.bucket_date + INTERVAL '1 day')::timestamp AT TIME ZONE 'UTC')
        ORDER BY ssh.shipment_id, ssh.changed_at DESC, ssh.id DESC
    ) latest
    WHERE latest.office_id = $3::uuid
      AND latest.to_status IN ('NEW', 'ACCEPTED', 'PROCESSED')
) snapshot ON TRUE
ORDER BY b.bucket_date ASC
"#;

const DELIVERIES_SERIES_SQL: &str = r#"
WITH buckets AS (
    SELECT GENERATE_SERIES($1::date, $2::date, INTERVAL '1 day')::date AS bucket_date
),
counts AS (
    SELECT DATE(timezone('UTC', changed_at)) AS bucket_date, COUNT(*)::bigint AS value
    FROM shipment_status_history
    WHERE office_id = $3::uuid
      AND to_status = 'DELIVERED'
      AND changed_at >= $4::timestamptz
      AND changed_at < $5::timestamptz
    GROUP BY 1
)
SELECT
    TO_CHAR(b.bucket_date, 'YYYY-MM-DD') AS bucket_start,
    COALESCE(c.value, 0)::bigint AS value
FROM buckets b
LEFT JOIN counts c USING (bucket_date)
ORDER BY b.bucket_date ASC
"#;

impl AnalyticsRepo {
    pub async fn admin_overview(
        db: &DatabaseConnection,
        span: AnalyticsSpan,
    ) -> Result<AdminOverviewMetrics, AnalyticsQueryError> {
        let window = SpanWindow::new(span);
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            ADMIN_OVERVIEW_SQL,
            vec![
                window.current_start.to_rfc3339().into(),
                window.current_end_exclusive.to_rfc3339().into(),
                window.previous_start.to_rfc3339().into(),
            ],
        );
        let row = db
            .query_one(statement)
            .await?
            .ok_or_else(|| DbErr::Custom("admin overview row missing".to_string()))?;

        let total_employees: i64 = row.try_get("", "total_employees")?;
        let assigned_employees: i64 = row.try_get("", "assigned_employees")?;

        Ok(AdminOverviewMetrics {
            total_shipments: row.try_get("", "total_shipments")?,
            shipments_vs_last_period: diff(
                row.try_get("", "current_shipments")?,
                row.try_get("", "previous_shipments")?,
            ),
            shipments_timeseries: daily_series(db, DAILY_SHIPMENTS_SERIES_SQL, &window).await?,
            total_clients: row.try_get("", "total_clients")?,
            clients_vs_last_period: diff(
                row.try_get("", "current_clients")?,
                row.try_get("", "previous_clients")?,
            ),
            clients_timeseries: daily_series(db, DAILY_CLIENTS_SERIES_SQL, &window).await?,
            total_offices: row.try_get("", "total_offices")?,
            offices_vs_last_period: diff(
                row.try_get("", "current_offices")?,
                row.try_get("", "previous_offices")?,
            ),
            offices_timeseries: daily_series(db, DAILY_OFFICES_SERIES_SQL, &window).await?,
            total_employees,
            assigned_employees,
            unassigned_employees: total_employees.saturating_sub(assigned_employees),
            employees_timeseries: daily_series(db, DAILY_EMPLOYEES_SERIES_SQL, &window).await?,
        })
    }

    pub async fn employee_overview(
        db: &DatabaseConnection,
        office_id: Uuid,
        span: AnalyticsSpan,
    ) -> Result<EmployeeOverviewMetrics, AnalyticsQueryError> {
        let window = SpanWindow::new(span);
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            EMPLOYEE_OVERVIEW_SQL,
            vec![
                office_id.to_string().into(),
                window.current_start.to_rfc3339().into(),
                window.today_start.to_rfc3339().into(),
                window.tomorrow_start.to_rfc3339().into(),
                window.yesterday_start.to_rfc3339().into(),
            ],
        );
        let row = db
            .query_one(statement)
            .await?
            .ok_or_else(|| DbErr::Custom("employee overview row missing".to_string()))?;

        let active_shipments: i64 = row.try_get("", "active_shipments")?;
        let previous_active_shipments: i64 = row.try_get("", "previous_active_shipments")?;
        let pending_shipments: i64 = row.try_get("", "pending_shipments")?;
        let previous_pending_shipments: i64 = row.try_get("", "previous_pending_shipments")?;
        let deliveries_today: i64 = row.try_get("", "deliveries_today")?;
        let deliveries_yesterday: i64 = row.try_get("", "deliveries_yesterday")?;

        Ok(EmployeeOverviewMetrics {
            active_shipments,
            active_vs_last_period: diff(active_shipments, previous_active_shipments),
            active_timeseries: active_series(db, &window, office_id).await?,
            pending_shipments,
            pending_vs_last_period: diff(pending_shipments, previous_pending_shipments),
            pending_timeseries: pending_series(db, &window, office_id).await?,
            deliveries_today,
            deliveries_vs_last_period: diff(deliveries_today, deliveries_yesterday),
            deliveries_timeseries: deliveries_series(db, &window, office_id).await?,
        })
    }

    pub fn empty_employee_overview(span: AnalyticsSpan) -> EmployeeOverviewMetrics {
        let window = SpanWindow::new(span);
        let zeros = window
            .bucket_dates()
            .into_iter()
            .map(|bucket_date| TimeseriesPoint {
                bucket_start: bucket_date.format("%Y-%m-%d").to_string(),
                value: 0,
            })
            .collect::<Vec<_>>();

        EmployeeOverviewMetrics {
            active_shipments: 0,
            active_vs_last_period: 0,
            active_timeseries: zeros.clone(),
            pending_shipments: 0,
            pending_vs_last_period: 0,
            pending_timeseries: zeros.clone(),
            deliveries_today: 0,
            deliveries_vs_last_period: 0,
            deliveries_timeseries: zeros,
        }
    }
}

#[derive(Debug, Clone)]
struct SpanWindow {
    start_date: NaiveDate,
    end_date: NaiveDate,
    current_start: DateTime<Utc>,
    current_end_exclusive: DateTime<Utc>,
    previous_start: DateTime<Utc>,
    today_start: DateTime<Utc>,
    tomorrow_start: DateTime<Utc>,
    yesterday_start: DateTime<Utc>,
}

impl SpanWindow {
    fn new(span: AnalyticsSpan) -> Self {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(span.days() - 1);
        let current_start = at_midnight(start_date);
        let current_end_exclusive = at_midnight(end_date + Duration::days(1));
        let previous_start = current_start - Duration::days(span.days());
        let today_start = at_midnight(end_date);
        let tomorrow_start = today_start + Duration::days(1);
        let yesterday_start = today_start - Duration::days(1);

        Self {
            start_date,
            end_date,
            current_start,
            current_end_exclusive,
            previous_start,
            today_start,
            tomorrow_start,
            yesterday_start,
        }
    }

    fn bucket_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let mut cursor = self.start_date;
        while cursor <= self.end_date {
            dates.push(cursor);
            cursor += Duration::days(1);
        }
        dates
    }
}

async fn daily_series(
    db: &DatabaseConnection,
    sql: &str,
    window: &SpanWindow,
) -> Result<Vec<TimeseriesPoint>, AnalyticsQueryError> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                window.start_date.format("%Y-%m-%d").to_string().into(),
                window.end_date.format("%Y-%m-%d").to_string().into(),
                window.current_start.to_rfc3339().into(),
                window.current_end_exclusive.to_rfc3339().into(),
            ],
        ))
        .await?;
    rows.into_iter()
        .map(|row| {
            Ok(TimeseriesPoint {
                bucket_start: row.try_get("", "bucket_start")?,
                value: row.try_get("", "value")?,
            })
        })
        .collect::<Result<Vec<_>, DbErr>>()
        .map_err(AnalyticsQueryError::from)
}

async fn active_series(
    db: &DatabaseConnection,
    window: &SpanWindow,
    office_id: Uuid,
) -> Result<Vec<TimeseriesPoint>, AnalyticsQueryError> {
    office_snapshot_series(db, ACTIVE_SERIES_SQL, window, office_id).await
}

async fn pending_series(
    db: &DatabaseConnection,
    window: &SpanWindow,
    office_id: Uuid,
) -> Result<Vec<TimeseriesPoint>, AnalyticsQueryError> {
    office_snapshot_series(db, PENDING_SERIES_SQL, window, office_id).await
}

async fn office_snapshot_series(
    db: &DatabaseConnection,
    sql: &str,
    window: &SpanWindow,
    office_id: Uuid,
) -> Result<Vec<TimeseriesPoint>, AnalyticsQueryError> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                window.start_date.format("%Y-%m-%d").to_string().into(),
                window.end_date.format("%Y-%m-%d").to_string().into(),
                office_id.to_string().into(),
            ],
        ))
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TimeseriesPoint {
                bucket_start: row.try_get("", "bucket_start")?,
                value: row.try_get("", "value")?,
            })
        })
        .collect::<Result<Vec<_>, DbErr>>()
        .map_err(AnalyticsQueryError::from)
}

async fn deliveries_series(
    db: &DatabaseConnection,
    window: &SpanWindow,
    office_id: Uuid,
) -> Result<Vec<TimeseriesPoint>, AnalyticsQueryError> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            DELIVERIES_SERIES_SQL,
            vec![
                window.start_date.format("%Y-%m-%d").to_string().into(),
                window.end_date.format("%Y-%m-%d").to_string().into(),
                office_id.to_string().into(),
                window.current_start.to_rfc3339().into(),
                window.current_end_exclusive.to_rfc3339().into(),
            ],
        ))
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TimeseriesPoint {
                bucket_start: row.try_get("", "bucket_start")?,
                value: row.try_get("", "value")?,
            })
        })
        .collect::<Result<Vec<_>, DbErr>>()
        .map_err(AnalyticsQueryError::from)
}

fn diff(current: i64, previous: i64) -> i64 {
    current - previous
}

fn at_midnight(date: NaiveDate) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(
        date.and_hms_opt(0, 0, 0).expect("midnight should be valid"),
        Utc,
    )
}
