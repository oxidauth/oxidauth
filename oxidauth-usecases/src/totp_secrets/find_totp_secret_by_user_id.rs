use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::find_role_by_id::*};
use oxidauth_repository::roles::select_role_by_id::SelectRoleByIdQuery;

pub struct FindTOTPSecretByUserIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    roles: T,
}

impl<T> FindTOTPSecretByUserIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindTOTPSecretByUserId>
    for FindTOTPSecretByUserIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_role_by_id_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a FindTOTPSecretByUserId,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
