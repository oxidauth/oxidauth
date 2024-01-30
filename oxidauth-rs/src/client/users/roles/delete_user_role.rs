use uuid::Uuid;

use oxidauth_http::{
    response::Response,
    server::api::v1::users::roles::delete_user_role::DeleteUserRoleRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "delete_user_role";

impl Client {
    pub async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
        where
            T: Into<Uuid>,
            R: Into<Uuid>,
    {
        let user_id = user_id.into();
        let role_id = role_id.into();

        let resp: Response<DeleteUserRoleRes> = self
            .delete(
                &format!("/users/{}/roles/{}", user_id, role_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(user_res)
    }
}
