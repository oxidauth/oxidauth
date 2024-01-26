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
    async fn update_role<T, U>(
        &self,
        role_id: T,
        role: U,
    ) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid>,
        U: Into<UpdateRoleReq>,
    {
        let role_id = role_id.into();
        let role = role.into();

        let resp: Response<UpdateRoleRes> = self
            .get(
                &format!("/roles/{}", role_id),
                role,
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
