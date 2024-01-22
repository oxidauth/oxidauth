use async_trait::async_trait;
use oxidauth_kernel::{
    auth::authenticate::{AuthenticateParams, AuthenticateResponse},
    error::BoxedError,
    service::Service,
};
use oxidauth_repository::authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery;

pub struct AuthenticateUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    authority_by_strategy: T,
}

impl<T> AuthenticateUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    pub fn new(authority_by_strategy: T) -> Self {
        Self {
            authority_by_strategy,
        }
    }
}

#[async_trait]
impl<'a, T> Service<&'a AuthenticateParams> for AuthenticateUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a AuthenticateParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_strategy
            .call(&params.into())
            .await?;

        todo!()
    }
}

pub async fn authenticate(
    params: &AuthenticateParams,
) -> Result<AuthenticateResponse, String> {
    //     let authority = authority_by_strategy(db, &request.strategy)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let authenticator = authority_factory(&authority, &request.strategy)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let user_identifier = authenticator
    //         .user_identifier_from_request(request.params.clone())
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let user_authority = user_authority_by_user_identifier(db, user_identifier)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     authenticator
    //         .authenticate(
    //             request.params,
    //             &user_authority,
    //         )
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let result = jwt_and_refresh_token(
    //         db,
    //         &authority,
    //         user_authority.user_id,
    //     )
    //     .await
    //     .map_err(|err| err.to_string())?;
    //
    //     Ok(result)
    todo!()
}
