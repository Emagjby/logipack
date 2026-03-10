use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub report_name: String,
    pub generated_at: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<JsonValue>>,
}
