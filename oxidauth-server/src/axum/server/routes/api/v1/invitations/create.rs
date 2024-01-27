use axum::{response::IntoResponse, Extension, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{
    server::routes::api::v1::users::{
        all::UserRow,
        create::{user_create, UserCreateRow},
    },
    Response,
};

use super::InvitationRow;

#[derive(Debug, Deserialize)]
pub struct InvitationCreateReq {
    pub user: UserCreateRow,
}

#[derive(Debug, Serialize)]
pub struct InvivationCreateRes {
    pub user: UserRow,
    pub invitation: InvitationRow,
}

pub async fn handle(
    Extension(db): Extension<PgPool>,
    Json(params): Json<InvitationCreateReq>,
) -> impl IntoResponse {
    let mut user = params.user;

    user.status
        .replace("invited".into());

    if user.kind.is_none() {
        user.kind
            .replace("human".into());
    }

    if user.profile.is_none() {
        user.profile
            .replace(serde_json::json!({}));
    }

    let mut db = db.acquire().await.unwrap();

    let user = match user_create(&mut db, user).await {
        Ok(user) => user,
        Err(err) => return Response::fail(err.to_string()).json(),
    };

    let expires_at = Utc::now()
        + Duration::from_std(std::time::Duration::from_secs(60 * 60 * 72))
            .unwrap();

    let invitation = InvitationCreateRow {
        user_id: user.id,
        expires_at,
    };

    let invitation = match invitation_create(&mut db, invitation).await {
        Ok(invitation) => invitation,
        Err(err) => return Response::fail(err.to_string()).json(),
    };

    Response::success(InvivationCreateRes { user, invitation }).json()
}

const QUERY: &str = r#"
    INSERT INTO invitations (
        user_id,
        expires_at
    ) VALUES ($1, $2)
    RETURNING *
"#;

pub async fn invitation_create(
    db: &mut PgConnection,
    invitation: InvitationCreateRow,
) -> Result<InvitationRow, sqlx::Error> {
    let result = sqlx::query_as::<_, InvitationRow>(QUERY)
        .bind(invitation.user_id)
        .bind(invitation.expires_at)
        .fetch_one(db)
        .await?;

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationCreateRow {
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}
