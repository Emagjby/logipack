use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use core_application::{actor::ActorContext, roles::Role};

use crate::auth::claims::Claims;
use crate::config::AuthMode;
use crate::state::AppState;

use core_data::entity::{employee_offices, employees, user_roles, users};

#[async_trait]
impl FromRequestParts<AppState> for ActorContext {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let sub = match state.auth_mode {
            AuthMode::DevSecret => parts
                .headers
                .get("x-dev-user-sub")
                .and_then(|v| v.to_str().ok())
                .ok_or((StatusCode::UNAUTHORIZED, "Missing x-dev-user-sub header"))?
                .to_string(),

            AuthMode::Auth0 => {
                let claims = parts
                    .extensions
                    .get::<Claims>()
                    .ok_or((StatusCode::UNAUTHORIZED, "Missing JWT claims"))?;

                claims.sub.clone()
            }
        };

        resolve_actor(&state.db, state.auth_mode, &sub)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid actor"))
    }
}

async fn resolve_actor(
    db: &DatabaseConnection,
    auth_mode: AuthMode,
    sub: &str,
) -> anyhow::Result<ActorContext> {
    // Resolve user
    let user = match auth_mode {
        AuthMode::DevSecret => {
            users::Entity::find()
                .filter(users::Column::Email.eq(sub))
                .one(db)
                .await?
        }

        AuthMode::Auth0 => {
            users::Entity::find()
                .filter(users::Column::Auth0Sub.eq(sub))
                .one(db)
                .await?
        }
    }
    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let user_id = user.id;

    // Resolve roles
    let roles = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .find_also_related(core_data::entity::roles::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|(_user_role, role)| {
            let role = role.ok_or_else(|| anyhow::anyhow!("Role not found"))?;
            match role.name.as_str() {
                "admin" => Ok(Role::Admin),
                "employee" => Ok(Role::Employee),
                _ => Err(anyhow::anyhow!("Unknown role")),
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    // Resolve employee
    let employee = employees::Entity::find()
        .filter(employees::Column::UserId.eq(user_id))
        .one(db)
        .await?;

    let (employee_id, allowed_office_ids) = if let Some(emp) = employee {
        let offices = employee_offices::Entity::find()
            .filter(employee_offices::Column::EmployeeId.eq(emp.id))
            .all(db)
            .await?
            .into_iter()
            .map(|eo| eo.office_id)
            .collect();

        (Some(emp.id), offices)
    } else {
        (None, vec![])
    };

    Ok(ActorContext {
        user_id,
        sub: sub.to_string(),
        roles,
        employee_id,
        allowed_office_ids,
    })
}

pub async fn resolve_actor_for_me(
    db: &DatabaseConnection,
    auth_mode: AuthMode,
    sub: &str,
) -> anyhow::Result<ActorContext> {
    resolve_actor(db, auth_mode, sub).await
}
