pub use sea_orm_migration::prelude::*;

mod m2026_01_13_init;
mod m2026_01_26_add_auth0_sub_to_users;
mod m2026_01_27_email_nullable;
mod m2026_01_27_password_hash_nullable;
mod m2026_02_13_soft_delete;
mod m2026_02_17_user_name;
mod m2026_03_07_create_audit_events;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2026_01_13_init::Migration),
            Box::new(m2026_01_26_add_auth0_sub_to_users::Migration),
            Box::new(m2026_01_27_password_hash_nullable::Migration),
            Box::new(m2026_01_27_email_nullable::Migration),
            Box::new(m2026_02_13_soft_delete::Migration),
            Box::new(m2026_02_17_user_name::Migration),
            Box::new(m2026_03_07_create_audit_events::Migration),
        ]
    }

    fn migration_table_name() -> DynIden {
        Alias::new("core_data_seaql_migrations").into_iden()
    }
}
