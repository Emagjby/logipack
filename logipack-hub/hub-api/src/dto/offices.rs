use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeDto {
    pub id: String,
    pub name: String,
    pub city: String,
    pub address: String,

    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOfficeRequest {
    pub name: String,
    pub city: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOfficeResponse {
    pub office: OfficeDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListOfficesResponse {
    pub offices: Vec<OfficeDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOfficeResponse {
    pub office: OfficeDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOfficeRequest {
    pub name: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOfficeResponse {
    pub office: OfficeDto,
}
