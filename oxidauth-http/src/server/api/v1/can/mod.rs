use axum::extract::State;
use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use oxidauth_permission::parse::parse;
use oxidauth_permission::validate;
use serde::{Deserialize, Serialize};

use crate::middleware::permission_extractor::ExtractEntitlements;
use crate::provider::Provider;
use crate::response::Response;

pub fn router() -> Router<Provider> {
    Router::new().route("/:permission", get(can))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanReq {
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanRes {}

#[tracing::instrument(name = "can_handler")]
async fn can(
    State(_): State<Provider>,
    Path(CanReq { permission }): Path<CanReq>,
    ExtractEntitlements(permissions): ExtractEntitlements,
) -> impl IntoResponse {
    let challenge = match parse(&permission) {
        Ok(permission) => permission,
        Err(err) => return Response::<bool>::fail().error(err.to_string()),
    };

    match validate(&challenge, &permissions) {
        Ok(true) => Response::success()
            .payload(true)
            .notice("yes you can"),
        Ok(false) => Response::success()
            .payload(false)
            .warning("no you can't"),
        Err(err) => Response::fail().error(err.to_string()),
    }
}
