use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::find_role_by_name::*};
use oxidauth_repository::roles::select_role_by_name::SelectRoleByNameQuery;

pub struct FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    roles: T,
}

impl<T> FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindRoleByName> for FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_role_by_name_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a FindRoleByName,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
