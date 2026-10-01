use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::delete_role::DeleteRoleRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "delete_role";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait DeleteRoleTrait {
    async fn delete_role<T>(&self, role_id: T) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_role<T>(&self, role_id: T) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let role_id = role_id.into();

        let resp: Response<DeleteRoleRes> = self
            .delete(&format!("/roles/{}", role_id), None::<Uuid>)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteRoleTrait for ClientMock {
    async fn delete_role<T>(&self, role_id: T) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.delete_role_fn.clone() else {
            panic!("delete_role not defined for mock client");
        };

        return func(role_id.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role};

    #[tokio::test]
    async fn delete_role_route_contract() {
        let role_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/roles/{role_id}"),
            ("role", "delete_role"),
            json!({ "role": role() }),
            move |client| {
                async move {
                    client
                        .delete_role(role_id)
                        .await
                }
            },
        )
        .await;
    }
}
