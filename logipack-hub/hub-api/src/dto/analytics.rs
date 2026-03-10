use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TimeseriesPointDto {
    pub bucket_start: String,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminOverviewResponse {
    pub total_shipments: i64,
    pub shipments_vs_last_period: i64,
    pub shipments_timeseries: Vec<TimeseriesPointDto>,
    pub total_clients: i64,
    pub clients_vs_last_period: i64,
    pub clients_timeseries: Vec<TimeseriesPointDto>,
    pub total_offices: i64,
    pub offices_vs_last_period: i64,
    pub offices_timeseries: Vec<TimeseriesPointDto>,
    pub total_employees: i64,
    pub assigned_employees: i64,
    pub unassigned_employees: i64,
    pub employees_timeseries: Vec<TimeseriesPointDto>,
}

#[derive(Debug, Serialize)]
pub struct EmployeeOverviewResponse {
    pub active_shipments: i64,
    pub active_vs_last_period: i64,
    pub active_timeseries: Vec<TimeseriesPointDto>,
    pub pending_shipments: i64,
    pub pending_vs_last_period: i64,
    pub pending_timeseries: Vec<TimeseriesPointDto>,
    pub deliveries_today: i64,
    pub deliveries_vs_last_period: i64,
    pub deliveries_timeseries: Vec<TimeseriesPointDto>,
}
