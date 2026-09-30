use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::update_role::{UpdateRoleReq, UpdateRoleRes};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "update_role";

#[async_trait]
pub trait UpdateRoleTrait {
    async fn update_role<T, U>(&self, role_id: T, role: U) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_role<T, U>(&self, role_id: T, role: U) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send,
    {
        let role_id = role_id.into();
        let role = role.into();

        let resp: Response<UpdateRoleRes> = self
            .put(&format!("/roles/{}", role_id), role)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl UpdateRoleTrait for ClientMock {
    async fn update_role<T, U>(&self, role_id: T, role: U) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send,
    {
        let Some(func) = self.update_role_fn.clone() else {
            panic!("update_role not defined for mock client");
        };

        return func(role_id.into(), role.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::roles::update_role::UpdateRole;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role};

    #[tokio::test]
    async fn update_role_route_contract() {
        let role_id = Uuid::new_v4();

        contract(
            "PUT",
            &format!("/api/v1/roles/{role_id}"),
            ("role", "update_role"),
            json!({ "role": role() }),
            move |client| {
                async move {
                    client
                        .update_role(
                            role_id,
                            UpdateRoleReq {
                                role: UpdateRole {
                                    role_id: None,
                                    name: "moderator".to_string(),
                                },
                            },
                        )
                        .await
                }
            },
        )
        .await;
    }
}
