use axum::{
    extract::Path, response::IntoResponse, routing::get, Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{axum::Response, jwt::ExtractEntitlements};

use super::permissions::Permission;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/:permission", get(can))
        .layer(Extension(database.clone()))
}

#[derive(Serialize, Deserialize)]
pub struct CanReq {
    pub permission: String,
}

#[derive(Serialize, Deserialize)]
pub struct CanRes {}

async fn can(
    // Extension(db): Extension<PgPool>,
    Path(CanReq { permission }): Path<CanReq>,
    ExtractEntitlements(mut permissions): ExtractEntitlements,
) -> impl IntoResponse {
    let permission: Permission = permission.try_into().unwrap();

    let result = permission.can(&mut permissions);

    Response::<_, ()>::success(result).json()
}
