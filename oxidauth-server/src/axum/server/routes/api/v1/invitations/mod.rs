pub mod accept;
pub mod create;
pub mod delete;
pub mod find;

use axum::{
    routing::{get, post, put},
    Extension, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", post(create::handle))
        .route("/:invitation_id", put(accept::handle))
        .route("/:invitation_id", get(find::handle))
        .layer(Extension(database.clone()))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvitationRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
