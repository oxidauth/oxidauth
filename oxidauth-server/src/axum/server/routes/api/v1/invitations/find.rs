use axum::{extract::Path, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{
    server::routes::api::v1::users::{
        all::UserRow,
        by_id::{self, user_by_id},
        create::user_create,
    },
    Response,
};

use super::InvitationRow;

#[derive(Debug, Deserialize)]
pub struct InvitationFindReq {
    pub invitation_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct InvivationFindRes {
    pub user: UserRow,
    pub invitation: InvitationRow,
}

pub async fn handle(
    Extension(db): Extension<PgPool>,
    Path(params): Path<InvitationFindReq>,
) -> impl IntoResponse {
    let mut db = db.acquire().await.unwrap();

    let invitation = match invitation_find(&mut db, params.invitation_id).await {
        Ok(invitation) => invitation,
        Err(err) => return Response::fail(err.to_string()).json(),
    };

    let user = match user_by_id(&mut db, invitation.user_id).await {
        Ok(user) => user,
        Err(err) => return Response::fail(err.to_string()).json(),
    };

    Response::success(InvivationFindRes { user, invitation }).json()
}

const QUERY: &str = r#"
    SELECT *
    FROM invitations
    WHERE id = $1
"#;

pub async fn invitation_find(
    db: &mut PgConnection,
    invitation_id: Uuid,
) -> Result<InvitationRow, sqlx::Error> {
    let result = sqlx::query_as::<_, InvitationRow>(QUERY)
        .bind(invitation_id)
        .fetch_one(db)
        .await?;

    Ok(result)
}
