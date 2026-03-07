use chrono::{DateTime, FixedOffset};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde_json::Value as JsonValue;
use thiserror::Error;
use uuid::Uuid;

use crate::entity::audit_events;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCursor {
    pub occurred_at: DateTime<FixedOffset>,
    pub id: Uuid,
}

impl AuditCursor {
    pub fn encode(&self) -> String {
        format!("{}|{}", self.occurred_at.to_rfc3339(), self.id)
    }

    pub fn decode(raw: &str) -> Result<Self, AuditRepoError> {
        let (occurred_at, id) = raw
            .split_once('|')
            .ok_or_else(|| AuditRepoError::InvalidCursor("missing delimiter".into()))?;

        let occurred_at = DateTime::parse_from_rfc3339(occurred_at)
            .map_err(|_| AuditRepoError::InvalidCursor("invalid timestamp".into()))?;
        let id = id
            .parse::<Uuid>()
            .map_err(|_| AuditRepoError::InvalidCursor("invalid uuid".into()))?;

        Ok(Self { occurred_at, id })
    }
}

#[derive(Debug, Clone)]
pub struct InsertAuditEvent {
    pub id: Uuid,
    pub occurred_at: DateTime<FixedOffset>,
    pub actor_user_id: Option<Uuid>,
    pub actor_display_name: Option<String>,
    pub action_key: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub entity_label: Option<String>,
    pub office_id: Option<Uuid>,
    pub office_label: Option<String>,
    pub target_route: Option<String>,
    pub metadata_json: Option<JsonValue>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaginatedAuditEvents {
    pub rows: Vec<audit_events::Model>,
    pub next_cursor: Option<AuditCursor>,
    pub has_next: bool,
}

#[derive(Debug, Error)]
pub enum AuditRepoError {
    #[error("db error: {0}")]
    Db(#[from] DbErr),
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),
}

pub struct AuditRepo;

impl AuditRepo {
    pub async fn insert_event(
        db: &DatabaseConnection,
        event: InsertAuditEvent,
    ) -> Result<audit_events::Model, AuditRepoError> {
        let model = audit_events::ActiveModel {
            id: Set(event.id),
            occurred_at: Set(event.occurred_at),
            actor_user_id: Set(event.actor_user_id),
            actor_display_name: Set(event.actor_display_name),
            action_key: Set(event.action_key),
            entity_type: Set(event.entity_type),
            entity_id: Set(event.entity_id),
            entity_label: Set(event.entity_label),
            office_id: Set(event.office_id),
            office_label: Set(event.office_label),
            target_route: Set(event.target_route),
            metadata_json: Set(event.metadata_json),
            request_id: Set(event.request_id),
        };

        Ok(model.insert(db).await?)
    }

    pub async fn list_paginated(
        db: &DatabaseConnection,
        limit: u64,
        cursor: Option<&AuditCursor>,
    ) -> Result<PaginatedAuditEvents, AuditRepoError> {
        let limit = limit.max(1);
        let fetch_limit = limit + 1;

        let mut query = audit_events::Entity::find()
            .order_by_desc(audit_events::Column::OccurredAt)
            .order_by_desc(audit_events::Column::Id);

        if let Some(cursor) = cursor {
            query = query.filter(
                Condition::any()
                    .add(audit_events::Column::OccurredAt.lt(cursor.occurred_at))
                    .add(
                        Condition::all()
                            .add(audit_events::Column::OccurredAt.eq(cursor.occurred_at))
                            .add(audit_events::Column::Id.lt(cursor.id)),
                    ),
            );
        }

        let mut rows = query.limit(fetch_limit).all(db).await?;
        let has_next = rows.len() as u64 > limit;

        if has_next {
            rows.truncate(limit as usize);
        }

        let next_cursor = rows.last().map(|row| AuditCursor {
            occurred_at: row.occurred_at.fixed_offset(),
            id: row.id,
        });

        Ok(PaginatedAuditEvents {
            rows,
            next_cursor: if has_next { next_cursor } else { None },
            has_next,
        })
    }
}
