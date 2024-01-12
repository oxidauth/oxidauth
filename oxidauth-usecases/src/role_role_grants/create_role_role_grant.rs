use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, role_role_grants::create_role_role_grant::*,
    roles::find_role_by_id::FindRoleById,
};
use oxidauth_repository::{
    role_role_grants::insert_role_role_grant::InsertRoleRoleGrantQuery,
    roles::select_role_by_id::SelectRoleByIdQuery,
};

pub struct CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    role_role_grants: T,
    roles: R,
}

impl<T, R> CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    pub fn new(role_role_grants: T, roles: R) -> Self {
        Self {
            role_role_grants,
            roles,
        }
    }
}

#[async_trait]
impl<'a, T, R> Service<&'a CreateRoleRoleGrant> for CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    type Response = CreateRoleRoleGrantResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_role_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a CreateRoleRoleGrant,
    ) -> Result<Self::Response, Self::Error> {
        self.roles
            .call(&FindRoleById {
                role_id: req.parent_id,
            })
            .await?;

        let child = self.roles
            .call(&FindRoleById {
                role_id: req.child_id,
            })
            .await?;

        let grant = self.role_role_grants
            .call(req)
            .await?;

        Ok(CreateRoleRoleGrantResponse { child, grant })
    }
}
