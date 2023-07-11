use axum::{extract::Path, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{
    server::routes::api::v1::users::{
        all::UserRow,
        by_id::{self, user_by_id},
        create::user_create,
        update::{user_update, UserUpdateRow},
    },
    Response,
};

use super::{find::invitation_find, InvitationRow};

#[derive(Debug, Deserialize)]
pub struct InvitationAcceptPath {
    pub invitation_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct InvitationAcceptBody {
    pub user: UserUpdateRow,
}

#[derive(Debug, Serialize)]
pub struct InvitationAcceptRes {
    pub user: UserRow,
}

pub async fn handle(
    Extension(db): Extension<PgPool>,
    Path(path_params): Path<InvitationAcceptPath>,
    Json(body_params): Json<InvitationAcceptBody>,
) -> impl IntoResponse {
    let params = AcceptParams {
        invitation_id: path_params.invitation_id,
        user: body_params.user,
    };

    match accept(db, params).await {
        Ok(user) => Response::success(InvitationAcceptRes { user }).json(),
        Err(err) => Response::fail(err.to_string()).json(),
    }
}

pub async fn accept(db: PgPool, params: AcceptParams) -> Result<UserRow, sqlx::Error> {
    let mut conn = db.acquire().await?;

    let invitation = invitation_find(&mut conn, params.invitation_id).await?;

    let user = user_by_id(&mut conn, invitation.user_id).await?;

    let updates = UserUpdateRow {
        id: Some(user.id),
        email: params.user.email.or(user.email),
        first_name: params.user.first_name.or(user.first_name),
        last_name: params.user.last_name.or(user.last_name),
        profile: params.user.profile.or(Some(user.profile)),
        status: Some("active".into()),
    };

    let user = user_update(db, updates).await?;

    Ok(user)
}

#[derive(Debug)]
pub struct AcceptParams {
    pub invitation_id: Uuid,
    pub user: UserUpdateRow,
}
