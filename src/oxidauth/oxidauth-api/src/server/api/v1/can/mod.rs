use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use oxidauth_http::{Response, can::CanReq};
use oxidauth_permission::{parse::parse, validate};

use crate::{middleware::permission_extractor::ExtractEntitlements, provider::Provider};

pub fn router() -> Router<Provider> {
    Router::new().route("/{permission}", get(can))
}

#[tracing::instrument(name = "can_handler")]
async fn can(
    State(_): State<Provider>,
    Path(CanReq { permission }): Path<CanReq>,
    ExtractEntitlements(permissions): ExtractEntitlements,
) -> impl IntoResponse {
    let challenge = match parse(&permission) {
        Ok(permission) => permission,
        Err(err) => return Response::<bool>::bad_request().error(err.to_string()),
    };

    match validate(&challenge, &permissions) {
        Ok(true) => {
            Response::success()
                .payload(true)
                .notice("yes you can")
        },
        Ok(false) => {
            Response::success()
                .payload(false)
                .warning("no you can't")
        },
        Err(err) => Response::bad_request().error(err.to_string()),
    }
}
