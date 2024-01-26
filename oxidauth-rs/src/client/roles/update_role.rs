use uuid::Uuid;

use oxidauth_http::{
    response::Response,
    server::api::v1::roles::update_role::{UpdateRoleReq, UpdateRoleRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "update_role";

impl Client {
    async fn update_role(
        &self,
        role_id: Uuid,
        params: &UpdateRoleReq,
    ) -> Result<UpdateRoleRes, BoxedError> {
        let resp: Response<UpdateRoleRes> = self
            .get(
                &format!("/roles/{}", role_id),
                None::<UpdateRoleReq>,
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
