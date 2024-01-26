use uuid::Uuid;

use oxidauth_http::{
    response::Response,
    server::api::v1::roles::find_role_by_id::FindRoleByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_id";

impl Client {
    async fn find_role_by_id(
        &self,
        role_id: Uuid,
    ) -> Result<FindRoleByIdRes, BoxedError> {
        let resp: Response<FindRoleByIdRes> = self
            .get(
                &format!("/roles/{}", role_id),
                None::<Uuid>,
            )
            .await?;

        let role_res = resp
            .payload
            .ok_or(ClientError::new(
                ClientErrorKind::EmptyPayload(RESOURCE, METHOD),
                None,
            ))?;

        Ok(role_res)
    }
}
