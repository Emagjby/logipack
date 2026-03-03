use serde::{Deserialize, Serialize};

use crate::dto::offices::OfficeDto;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeDto {
    pub id: String,
    pub user_id: String,

    pub full_name: String,
    pub user_display_name: Option<String>,
    pub email: String,

    pub user: Option<UserDto>,
    pub offices: Option<Vec<OfficeDto>>,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeListItemDto {
    pub id: String,
    pub user_id: String,
    pub full_name: String,
    pub user_display_name: Option<String>,
    pub email: String,
    pub offices: Option<Vec<OfficeDto>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEmployeeRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEmployeeResponse {
    pub employee: EmployeeDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListEmployeesResponse {
    pub employees: Vec<EmployeeListItemDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEmployeeResponse {
    pub employee: EmployeeDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEmployeeRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEmployeeResponse {
    pub employee: EmployeeDto,
}
