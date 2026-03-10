use axum::{Json, http::StatusCode, response::IntoResponse};
use core_application::shipments::{
    change_status::ChangeStatusError, create::CreateShipmentError, timeline::TimelineError,
};
use core_application::users::ensure_user::EnsureUserError;
use core_application::users::me::MeError;
use core_data::repository::{
    analytics_repo::AnalyticsQueryError, reporting_repo::ReportingQueryError,
    shipments_repo::ShipmentSnapshotError,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    pub fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
            message: message.into(),
        }
    }

    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
        }
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(value: sea_orm::DbErr) -> Self {
        match value {
            sea_orm::DbErr::RecordNotFound(_) => {
                ApiError::not_found("not_found", "Record not found")
            }
            other => match other.sql_err() {
                Some(sea_orm::SqlErr::ForeignKeyConstraintViolation(_)) => {
                    ApiError::bad_request("invalid_reference", "Referenced entity not found")
                }
                Some(sea_orm::SqlErr::UniqueConstraintViolation(_)) => ApiError::bad_request(
                    "constraint_violation",
                    "Request violates uniqueness constraint",
                ),
                Some(_) => ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "db_error",
                    message: other.to_string(),
                },
                None => ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "db_error",
                    message: other.to_string(),
                },
            },
        }
    }
}

impl From<ShipmentSnapshotError> for ApiError {
    fn from(value: ShipmentSnapshotError) -> Self {
        match value {
            ShipmentSnapshotError::DbError(db) => db.into(),
        }
    }
}

impl From<ReportingQueryError> for ApiError {
    fn from(value: ReportingQueryError) -> Self {
        match value {
            ReportingQueryError::Db(err) => err.into(),
        }
    }
}

impl From<AnalyticsQueryError> for ApiError {
    fn from(value: AnalyticsQueryError) -> Self {
        match value {
            AnalyticsQueryError::Db(err) => err.into(),
        }
    }
}

impl From<CreateShipmentError> for ApiError {
    fn from(err: CreateShipmentError) -> Self {
        match err {
            CreateShipmentError::Forbidden => {
                ApiError::forbidden("forbidden", "you are not allowed to create shipments")
            }

            CreateShipmentError::DbError(db) => db.into(),

            CreateShipmentError::EventstoreError(e) => {
                ApiError::internal(format!("eventstore error: {e}"))
            }

            CreateShipmentError::EnsureStreamError(e) => {
                ApiError::internal(format!("stream error: {e}"))
            }

            CreateShipmentError::SnapshotError(e) => e.into(),
            CreateShipmentError::Audit(e) => ApiError::internal(format!("audit error: {e}")),
        }
    }
}

impl From<ChangeStatusError> for ApiError {
    fn from(err: ChangeStatusError) -> Self {
        match err {
            ChangeStatusError::Forbidden => {
                ApiError::forbidden("forbidden", "you are not allowed to change shipment status")
            }

            ChangeStatusError::Domain(e) => ApiError::bad_request(
                "domain_transition_error",
                format!("invalid status transition: {e:?}"),
            ),

            ChangeStatusError::SnapshotError(e) => ApiError::from(e),

            ChangeStatusError::DbError(db) => db.into(),

            ChangeStatusError::StreamError(e) => ApiError::internal(format!("stream error: {e}")),

            ChangeStatusError::EventstoreError(e) => {
                ApiError::internal(format!("eventstore error: {e}"))
            }
            ChangeStatusError::Audit(e) => ApiError::internal(format!("audit error: {e}")),
        }
    }
}

impl From<TimelineError> for ApiError {
    fn from(value: TimelineError) -> Self {
        match value {
            TimelineError::Read(e) => ApiError::internal(format!("eventstore read error: {e}")),
        }
    }
}

impl From<EnsureUserError> for ApiError {
    fn from(err: EnsureUserError) -> Self {
        match err {
            EnsureUserError::Validation(v) => ApiError::bad_request("invalid_user", v.to_string()),
            EnsureUserError::InvalidAuth0Sub => {
                ApiError::bad_request("invalid_auth0_sub", "Invalid auth0 subject identifier")
            }
            EnsureUserError::EmailAlreadyLinked => ApiError::conflict(
                "email_already_linked",
                "Email already linked to another account",
            ),
            EnsureUserError::UserNotFound => {
                ApiError::not_found("user_not_found", "User not found")
            }
            EnsureUserError::DbError(e) => ApiError::internal(e),
        }
    }
}

impl From<MeError> for ApiError {
    fn from(err: MeError) -> Self {
        match err {
            MeError::NotFound => {
                ApiError::not_found("user_not_provisioned", "User not provisioned")
            }
            MeError::DbError(e) => ApiError::internal(e.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ApiErrorBody {
            code: self.code,
            message: self.message,
        };

        (self.status, Json(body)).into_response()
    }
}
