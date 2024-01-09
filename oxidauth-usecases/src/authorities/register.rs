use oxidauth_kernel::{
    authorities::{
        find_authority_by_client_id::FindAuthorityByClientIdService,
        register::*,
    },
    service::Service,
    user_authority::user_authority_create::{
        UserAuthorityCreate, UserAuthorityCreateService,
    },
    users::user_create::CreateUserTrait,
};

use crate::dev_prelude::*;

pub struct RegisterUseCase<A, U, UA>
where
    A: FindAuthorityByClientIdService,
    U: CreateUserTrait,
    UA: UserAuthorityCreateService,
{
    authorities: A,
    user_create: U,
    user_authorities: UA,
}

impl<A, U, UA> RegisterUseCase<A, U, UA>
where
    A: FindAuthorityByClientIdService,
    U: CreateUserTrait,
    UA: UserAuthorityCreateService,
{
    pub fn new(authorities: A, user_create: U, user_authorities: UA) -> Self {
        Self {
            authorities,
            user_create,
            user_authorities,
        }
    }
}

#[async_trait]
impl<A, U, UA, P> Service<P> for RegisterUseCase<A, U, UA>
where
    A: FindAuthorityByClientIdService,
    U: CreateUserTrait,
    UA: UserAuthorityCreateService,
    P: RegisterParamsExtractor,
{
    type Response = ();

    type Error = RegisterError;

    async fn call(&self, params: P) -> Result<Self::Response, Self::Error> {
        let client_id = params
            .client_id()
            .map_err(|_| RegisterError {})?;
        let user_identifier = params
            .user_identifier()
            .map_err(|_| RegisterError {})?;
        let user_create = params
            .user_create()
            .map_err(|_| RegisterError {})?;
        let user_authority_params = params
            .user_authority_params()
            .map_err(|_| RegisterError {})?;

        // find authority
        let authority = self
            .authorities
            .find_authority_by_client_id(client_id)
            .await
            .map_err(|_| RegisterError {})?;

        // create user
        let user = self
            .user_create
            .create_user(&user_create)
            .await
            .map_err(|_| RegisterError {})?;

        // create user_authority
        let user_authority = self
            .user_authorities
            .create_user_authority(&UserAuthorityCreate {
                user_id: user.id,
                authority_id: authority.id,
                user_identifier,
                params: user_authority_params,
            })
            .await
            .map_err(|_| RegisterError {})?;

        Ok(())
    }
}

#[async_trait]
impl<A, U, UA, P> RegisterService<P> for RegisterUseCase<A, U, UA>
where
    A: FindAuthorityByClientIdService,
    U: CreateUserTrait,
    UA: UserAuthorityCreateService,
    P: RegisterParamsExtractor,
{
    async fn register(&self, params: P) -> Result<(), RegisterError> {
        let client_id = params
            .client_id()
            .map_err(|_| RegisterError {})?;
        let user_identifier = params
            .user_identifier()
            .map_err(|_| RegisterError {})?;
        let user_create = params
            .user_create()
            .map_err(|_| RegisterError {})?;
        let user_authority_params = params
            .user_authority_params()
            .map_err(|_| RegisterError {})?;

        // find authority
        let authority = self
            .authorities
            .find_authority_by_client_id(client_id)
            .await
            .map_err(|_| RegisterError {})?;

        // create user
        let user = self
            .user_create
            .create_user(&user_create)
            .await
            .map_err(|_| RegisterError {})?;

        // create user_authority
        let user_authority = self
            .user_authorities
            .create_user_authority(&UserAuthorityCreate {
                user_id: user.id,
                authority_id: authority.id,
                user_identifier,
                params: user_authority_params,
            })
            .await
            .map_err(|_| RegisterError {})?;

        Ok(())
    }
}
