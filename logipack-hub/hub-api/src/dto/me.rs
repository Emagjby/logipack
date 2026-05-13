use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub role: String,
    pub office_ids: Vec<String>,
    pub current_office_id: Option<String>,
    pub current_office_name: Option<String>,
    pub employee_id: Option<String>,
}
