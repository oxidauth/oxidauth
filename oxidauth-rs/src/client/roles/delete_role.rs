use uuid::Uuid;

use oxidauth_http::{
    response::Response,
    server::api::v1::roles::delete_role::DeleteRoleRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "delete_role";

impl Client {
    async fn delete_role<T>(
        &self,
        role_id: T,
    ) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid>,
    {
        let role_id = role_id.into();

        let resp: Response<DeleteRoleRes> = self
            .delete(
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
