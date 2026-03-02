use base64::Engine;
use core_domain::shipment::ShipmentStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::{clients::ClientDto, offices::OfficeDto};

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipmentListItem {
    pub id: String,
    pub client_id: String,
    pub current_status: String,
    pub current_office_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipmentDetail {
    pub id: String,
    pub client: ClientDto,
    pub current_status: String,
    pub current_office: Option<OfficeDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateShipmentRequest {
    pub client_id: Uuid,
    pub current_office_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateShipmentResponse {
    pub shipment_id: Uuid,
}

#[derive(Deserialize)]
pub struct ChangeStatusRequest {
    pub to_status: ShipmentStatus,
    pub to_office_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct TimelineItem {
    pub seq: i64,
    pub event_type: String,
    /// Strata Canonical Bytes encoded as base64.
    pub scb: String,
}

impl From<core_eventstore::adapter::read::StreamPackage> for TimelineItem {
    fn from(value: core_eventstore::adapter::read::StreamPackage) -> Self {
        Self {
            seq: value.seq,
            event_type: value.event_type,
            scb: base64::engine::general_purpose::STANDARD.encode(value.scb),
        }
    }
}

impl From<core_data::entity::shipments::Model> for ShipmentListItem {
    fn from(value: core_data::entity::shipments::Model) -> Self {
        Self {
            id: value.id.to_string(),
            client_id: value.client_id.to_string(),
            current_status: value.current_status,
            current_office_id: value.current_office_id.map(|id| id.to_string()),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

impl From<core_data::entity::shipments::Model> for ShipmentDetail {
    fn from(value: core_data::entity::shipments::Model) -> Self {
        Self {
            id: value.id.to_string(),
            client: ClientDto {
                id: value.client_id.to_string(),
                name: "".to_string(),
                email: None,
                phone: None,
                updated_at: value.updated_at.to_rfc3339(),
            },
            current_status: value.current_status,
            current_office: None,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}
