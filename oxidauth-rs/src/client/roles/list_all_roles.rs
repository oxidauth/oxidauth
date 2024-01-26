use oxidauth_http::{
    response::Response,
    server::api::v1::roles::list_all_roles::{ListAllRolesRes, ListAllRolesReq},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "list_all_roles";

impl Client {
    async fn list_all_roles<T>(
        &self,
        params: T,
    ) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq>,
    {
        let _params = params.into();

        let resp: Response<ListAllRolesRes> = self
            .get(
                "/roles",
                None::<ListAllRolesReq>,
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
