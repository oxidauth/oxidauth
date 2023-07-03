use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};

use crate::{authorities::AuthorityStrategy, axum::Response};

use super::all::AuthorityRow;

#[derive(Deserialize)]
pub struct AuthorityByStrategyReq {
    authority_strategy: AuthorityStrategy,
}

#[derive(Serialize)]
pub struct AuthorityByStrategyRes {
    authority: AuthorityRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<AuthorityByStrategyReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = authority_by_strategy(&mut db, &params.authority_strategy).await;

    match result {
        Ok(authority) => Response::success(AuthorityByStrategyRes { authority }).json(),
        Err(error) => Response::fail(format!("authority not found: {}", error.to_string())).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT *
    FROM authorities
    WHERE strategy = $1
"#;

pub async fn authority_by_strategy(
    db: &mut PgConnection,
    strategy: &AuthorityStrategy,
) -> Result<AuthorityRow, sqlx::Error> {
    let result = sqlx::query_as::<_, AuthorityRow>(QUERY)
        .bind(strategy)
        .fetch_one(db)
        .await?;

    Ok(result)
}
