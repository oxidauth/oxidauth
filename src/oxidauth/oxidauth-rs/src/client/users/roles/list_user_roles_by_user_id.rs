use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::roles::list_user_roles_by_user_id::ListUserRoleGrantsByUserIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "list_user_roles_by_user_id";

#[async_trait]
pub trait ListUserRolesByUserIdTrait {
    async fn list_user_roles_by_user_id<T>(
        &self,
        user_id: T,
    ) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl ListUserRolesByUserIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_user_roles_by_user_id<T>(
        &self,
        user_id: T,
    ) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let user_id = user_id.into();

        let resp: Response<ListUserRoleGrantsByUserIdRes> = self
            .get(&format!("/users/{}/roles", user_id), None::<()>)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListUserRolesByUserIdTrait for ClientMock {
    async fn list_user_roles_by_user_id<T>(
        &self,
        user_id: T,
    ) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_user_roles_by_user_id_fn
            .clone()
        else {
            panic!("list_user_roles_by_user_id not defined for mock client");
        };

        return func(user_id.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_role};

    #[tokio::test]
    async fn list_user_roles_by_user_id_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/users/{user_id}/roles"),
            ("user_role", "list_user_roles_by_user_id"),
            json!({ "user_role_grants": [user_role()] }),
            move |client| {
                async move {
                    client
                        .list_user_roles_by_user_id(user_id)
                        .await
                }
            },
        )
        .await;
    }
}
