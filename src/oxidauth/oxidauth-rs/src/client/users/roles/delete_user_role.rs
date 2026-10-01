use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::roles::delete_user_role::DeleteUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "delete_user_role";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait DeleteUserRoleTrait {
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteUserRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send,
    {
        let user_id = user_id.into();
        let role_id = role_id.into();

        let resp: Response<DeleteUserRoleRes> = self
            .delete(&format!("/users/{}/roles/{}", user_id, role_id), None::<()>)
            .await?;

        let user_role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteUserRoleTrait for ClientMock {
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_user_role_fn
            .clone()
        else {
            panic!("delete_user_role not defined for mock client");
        };

        return func(user_id.into(), role_id.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_role};

    #[tokio::test]
    async fn delete_user_role_route_contract() {
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/users/{user_id}/roles/{role_id}"),
            ("user_role", "delete_user_role"),
            json!({ "user_role": user_role() }),
            move |client| {
                async move {
                    client
                        .delete_user_role(user_id, role_id)
                        .await
                }
            },
        )
        .await;
    }
}
