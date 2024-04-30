use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::list_all_roles::*};
use oxidauth_repository::roles::select_all_roles::SelectAllRolesQuery;

pub struct ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    roles: T,
}

impl<T> ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ListAllRoles> for ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    type Response = Vec<Role>;
    type Error = BoxedError;

    #[tracing::instrument(name = "list_all_roles_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ListAllRoles,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
