use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuditEvents::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AuditEvents::ActorUserId).uuid().null())
                    .col(
                        ColumnDef::new(AuditEvents::ActorDisplayName)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(AuditEvents::ActionKey).string().not_null())
                    .col(ColumnDef::new(AuditEvents::EntityType).string().null())
                    .col(ColumnDef::new(AuditEvents::EntityId).string().null())
                    .col(ColumnDef::new(AuditEvents::EntityLabel).string().null())
                    .col(ColumnDef::new(AuditEvents::OfficeId).uuid().null())
                    .col(ColumnDef::new(AuditEvents::OfficeLabel).string().null())
                    .col(ColumnDef::new(AuditEvents::TargetRoute).string().null())
                    .col(
                        ColumnDef::new(AuditEvents::MetadataJson)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(AuditEvents::RequestId).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_events_occurred_at_id")
                    .table(AuditEvents::Table)
                    .col(AuditEvents::OccurredAt)
                    .col(AuditEvents::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_events_action_key")
                    .table(AuditEvents::Table)
                    .col(AuditEvents::ActionKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_events_entity")
                    .table(AuditEvents::Table)
                    .col(AuditEvents::EntityType)
                    .col(AuditEvents::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_events_office_id")
                    .table(AuditEvents::Table)
                    .col(AuditEvents::OfficeId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditEvents {
    Table,
    Id,
    OccurredAt,
    ActorUserId,
    ActorDisplayName,
    ActionKey,
    EntityType,
    EntityId,
    EntityLabel,
    OfficeId,
    OfficeLabel,
    TargetRoute,
    MetadataJson,
    RequestId,
}
