use serde::{Deserialize, Serialize};

use crate::dto::{employees::EmployeeDto, offices::OfficeDto};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignOfficeRequest {
    pub office_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListEmployeeOfficesResponse {
    pub employee_id: String,
    pub offices: Vec<OfficeDto>,
    pub office_ids: Vec<String>,
    pub employee: Option<EmployeeDto>,
    pub assigned_offices: Option<Vec<OfficeDto>>,
}
