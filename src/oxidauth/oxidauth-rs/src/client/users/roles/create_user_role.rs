use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::roles::create_user_role::CreateUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use serde::Serialize;
use uuid::Uuid;

use super::*;

#[derive(Debug, Serialize)]
pub struct CreateUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "create_user_role";

#[async_trait]
pub trait CreateUserRoleTrait {
    async fn create_user_role<T>(&self, params: T) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send;
}

#[async_trait]
impl CreateUserRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user_role<T>(&self, params: T) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send,
    {
        let CreateUserRole { user_id, role_id } = params.into();

        let resp: Response<CreateUserRoleRes> = self
            .post(&format!("/users/{}/roles/{}", user_id, role_id), None::<()>)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreateUserRoleTrait for ClientMock {
    async fn create_user_role<T>(&self, params: T) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_user_role_fn
            .clone()
        else {
            panic!("create_user_role not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_role};

    #[tokio::test]
    async fn create_user_role_route_contract() {
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        // the grant create returns the raw UserRole payload (no wrapper key)
        contract(
            "POST",
            &format!("/api/v1/users/{user_id}/roles/{role_id}"),
            ("user_role", "create_user_role"),
            user_role(),
            move |client| {
                async move {
                    client
                        .create_user_role(CreateUserRole { user_id, role_id })
                        .await
                }
            },
        )
        .await;
    }
}
