use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, QuerySelect, TransactionTrait,
};
use thiserror::Error;
use uuid::Uuid;

use crate::entity::{roles, user_roles, users};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("db error: {0}")]
    UserError(#[from] DbErr),
    #[error("user not found")]
    RecordNotFound,
    #[error("email already linked to another account")]
    EmailAlreadyLinked,
}

pub struct UserRepo;

impl UserRepo {
    pub async fn get_by_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<users::Model>, UserError> {
        let user = users::Entity::find_by_id(user_id).one(db).await?;
        Ok(user)
    }

    /// Finds a user by their Auth0 subject identifier.
    pub async fn get_by_auth0_sub(
        db: &DatabaseConnection,
        auth0_sub: &str,
    ) -> Result<Option<users::Model>, UserError> {
        let user = users::Entity::find()
            .filter(users::Column::Auth0Sub.eq(auth0_sub))
            .one(db)
            .await?;
        Ok(user)
    }

    /// Looks up a user by `auth0_sub` and returns their role name from
    /// the `user_roles` + `roles` join, if any.
    ///
    /// Returns:
    /// - `Ok(Some(role_name))` if the user exists and has a role assigned.
    /// - `Ok(Some(""))` if the user exists but has no role.
    /// - `Ok(None)` if no user with that `auth0_sub` exists.
    pub async fn get_role_by_auth0_sub(
        db: &DatabaseConnection,
        auth0_sub: &str,
    ) -> Result<Option<String>, UserError> {
        let user = users::Entity::find()
            .filter(users::Column::Auth0Sub.eq(auth0_sub))
            .one(db)
            .await?;

        let user = match user {
            Some(u) => u,
            None => return Ok(None),
        };

        // Find the first role via user_roles -> roles join
        let role_row = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .find_also_related(roles::Entity)
            .one(db)
            .await?;

        let role_name = role_row
            .and_then(|(_, role)| role)
            .map(|r| r.name)
            .unwrap_or_default();

        Ok(Some(role_name))
    }

    /// Finds a user by their email address (case-insensitive comparison).
    pub async fn get_by_email(
        db: &impl sea_orm::ConnectionTrait,
        email: &str,
    ) -> Result<Option<users::Model>, UserError> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(Some(email.to_lowercase())))
            .one(db)
            .await?;
        Ok(user)
    }

    /// Ensures a user row exists for the given Auth0 subject.
    ///
    /// Linking flow (inside a transaction):
    /// 1. Look up by `auth0_sub`. If found, update name/email if changed, return id.
    /// 2. If not found by sub, look up by `email` (with `FOR UPDATE` lock to prevent races):
    ///    a. If found and `auth0_sub` is NULL: link the identity (set auth0_sub), update name, return id.
    ///    b. If found and `auth0_sub` is set: return `EmailAlreadyLinked` error (must be a different
    ///    sub, since matching sub would have been caught in Step 1).
    /// 3. If not found at all: insert a new user row.
    ///
    /// Returns the user's internal UUID.
    pub async fn ensure_user_by_auth0_sub(
        db: &DatabaseConnection,
        auth0_sub: String,
        name: String,
        email: String,
    ) -> Result<Uuid, UserError> {
        let email_lower = email.to_lowercase();

        let txn = db.begin().await?;

        // Step 1: Try find by auth0_sub
        let by_sub = users::Entity::find()
            .filter(users::Column::Auth0Sub.eq(auth0_sub.clone()))
            .one(&txn)
            .await?;

        if let Some(user) = by_sub {
            let user_id = user.id;
            let name_changed = user.name != name;
            let email_changed = user.email.as_deref() != Some(&email_lower);

            if name_changed || email_changed {
                let mut active = user.into_active_model();
                if name_changed {
                    active.name = Set(name);
                }
                if email_changed {
                    active.email = Set(Some(email_lower));
                }
                active.update(&txn).await?;
            }

            txn.commit().await?;
            return Ok(user_id);
        }

        // Step 2: Try find by email (with row-level lock to prevent concurrent link races)
        let by_email = users::Entity::find()
            .filter(users::Column::Email.eq(Some(email_lower.clone())))
            .lock_exclusive()
            .one(&txn)
            .await?;

        if let Some(user) = by_email {
            match &user.auth0_sub {
                // 2a: auth0_sub is NULL -> link identity
                None => {
                    let user_id = user.id;
                    let mut active = user.into_active_model();
                    active.auth0_sub = Set(Some(auth0_sub));
                    if active.name.as_ref() != &name {
                        active.name = Set(name);
                    }
                    active.update(&txn).await?;
                    txn.commit().await?;
                    return Ok(user_id);
                }
                // 2b: auth0_sub is set (must be different since Step 1 would
                //     have matched if it were the same) -> conflict
                Some(_) => {
                    txn.rollback().await?;
                    return Err(UserError::EmailAlreadyLinked);
                }
            }
        }

        // Step 3: No existing user -> insert new
        let user_id = Uuid::new_v4();

        let new_user = users::ActiveModel {
            id: Set(user_id),
            name: Set(name),
            password_hash: Set(None),
            email: Set(Some(email_lower)),
            auth0_sub: Set(Some(auth0_sub)),
            created_at: Set(chrono::Utc::now().into()),
        };

        new_user.insert(&txn).await?;
        txn.commit().await?;

        Ok(user_id)
    }
}
