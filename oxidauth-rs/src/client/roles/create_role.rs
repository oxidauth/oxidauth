use oxidauth_http::{
    response::Response,
    server::api::v1::roles::create_role::{CreateRoleReq, CreateRoleRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "create_role";

impl Client {
    async fn create_role(
        &self,
        role: &CreateRoleReq,
    ) -> Result<CreateRoleRes, BoxedError> {
        let resp: Response<CreateRoleRes> = self
            .post("/roles", role)
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
