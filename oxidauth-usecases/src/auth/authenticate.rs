use async_trait::async_trait;
use oxidauth_kernel::{
    auth::{
        authenticate::{AuthenticateParams, AuthenticateResponse},
        Authenticator,
    },
    authorities::{Authority, AuthorityStrategy},
    error::BoxedError,
    service::Service,
};
use oxidauth_repository::{authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery, user_authorities::select_user_authorities_by_authority_id_and_user_identifier::SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery};

pub struct AuthenticateUseCase<T, U>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
{
    authority_by_strategy: T,
    user_authority: U,
}

impl<T, U> AuthenticateUseCase<T, U>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
{
    pub fn new(authority_by_strategy: T, user_authority: U) -> Self {
        Self {
            authority_by_strategy,
            user_authority,
        }
    }
}

#[async_trait]
impl<'a, T, U> Service<&'a AuthenticateParams> for AuthenticateUseCase<T, U>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
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

        let authenticator =
            build_authenticator(&authority, &params.strategy).await?;

        let user_identifier = authenticator
            .user_identifier_from_request(&params.params)
            .await?;

        let user_authority = todo!();

        todo!()
    }
}

pub async fn build_authenticator(
    authority: &Authority,
    strategy: &AuthorityStrategy,
) -> Result<Box<dyn Authenticator>, BoxedError> {
    todo!()
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
