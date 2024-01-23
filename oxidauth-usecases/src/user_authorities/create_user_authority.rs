use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, user_authorities::create_user_authority::*,
};
use oxidauth_repository::{
    authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery,
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
};

use crate::auth::register::build_registrar;

pub struct CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByStrategyQuery,
    U: InsertUserAuthorityQuery,
{
    authority_by_strategy: A,
    insert_user_authority: U,
}

impl<A, U> CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByStrategyQuery,
    U: InsertUserAuthorityQuery,
{
    pub fn new(authority_by_strategy: A, insert_user_authority: U) -> Self {
        Self {
            authority_by_strategy,
            insert_user_authority,
        }
    }
}

#[async_trait]
impl<'a, A, U> Service<&'a CreateUserAuthorityParams>
    for CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByStrategyQuery,
    U: InsertUserAuthorityQuery,
{
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_user_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a CreateUserAuthorityParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_strategy
            .call(&req.into())
            .await?;

        let registrar = build_registrar(&authority, &req.strategy).await?;

        let mut user_authority = registrar
            .user_authority_from_request(req.params.clone())
            .await?;

        user_authority.user_id = Some(req.user_id);

        self.insert_user_authority
            .call(&user_authority)
            .await
    }
}
