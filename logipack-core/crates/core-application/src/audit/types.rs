use chrono::{DateTime, FixedOffset};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditActionKey {
    ShipmentCreated,
    ShipmentStatusUpdated,
    OfficeCreated,
    OfficeUpdated,
    OfficeDeleted,
    ClientCreated,
    ClientUpdated,
    ClientDeleted,
    EmployeeCreated,
    EmployeeUpdated,
    EmployeeDeleted,
    EmployeeAssignedToOffice,
    EmployeeRemovedFromOffice,
}

impl AuditActionKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShipmentCreated => "shipment.created",
            Self::ShipmentStatusUpdated => "shipment.status_updated",
            Self::OfficeCreated => "office.created",
            Self::OfficeUpdated => "office.updated",
            Self::OfficeDeleted => "office.deleted",
            Self::ClientCreated => "client.created",
            Self::ClientUpdated => "client.updated",
            Self::ClientDeleted => "client.deleted",
            Self::EmployeeCreated => "employee.created",
            Self::EmployeeUpdated => "employee.updated",
            Self::EmployeeDeleted => "employee.deleted",
            Self::EmployeeAssignedToOffice => "employee.assigned_to_office",
            Self::EmployeeRemovedFromOffice => "employee.removed_from_office",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEntityType {
    Shipment,
    Office,
    Client,
    Employee,
    Role,
    User,
    System,
}

impl AuditEntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shipment => "shipment",
            Self::Office => "office",
            Self::Client => "client",
            Self::Employee => "employee",
            Self::Role => "role",
            Self::User => "user",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditEventInput {
    pub action_key: AuditActionKey,
    pub entity_type: Option<AuditEntityType>,
    pub entity_id: Option<String>,
    pub entity_label: Option<String>,
    pub office_id: Option<Uuid>,
    pub office_label: Option<String>,
    pub target_route: Option<String>,
    pub metadata_json: Option<JsonValue>,
    pub request_id: Option<String>,
    pub occurred_at: Option<DateTime<FixedOffset>>,
}
