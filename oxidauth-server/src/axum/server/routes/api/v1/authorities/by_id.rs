use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

use super::all::AuthorityRow;

#[derive(Deserialize)]
pub struct AuthorityByIDReq {
    authority_id: Uuid,
}

#[derive(Serialize)]
pub struct AuthorityByIDRes {
    authority: AuthorityRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<AuthorityByIDReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = authority_by_id(&mut db, params.authority_id).await;

    match result {
        Ok(authority) => Response::success(AuthorityByIDRes { authority }).json(),
        Err(error) => Response::fail(format!("authority not found: {}", error.to_string())).json(),
    }
}

pub async fn authority_by_id(
    db: &mut PgConnection,
    authority_id: Uuid,
) -> Result<AuthorityRow, sqlx::Error> {
    let result = sqlx::query_as::<_, AuthorityRow>(QUERY)
        .bind(authority_id)
        .fetch_one(db)
        .await?;

    Ok(result)
}

pub const QUERY: &str = r#"
    SELECT *
    FROM authorities
    WHERE id = $1
"#;
