use crate::dev_prelude::*;

#[async_trait]
pub trait RegisterService<P>: Send + Sync + 'static
where
    P: RegisterParamsExtractor,
{
    async fn register(&self, params: P) -> Result<(), RegisterError>;
}

#[derive(Debug)]
pub struct RegisterError {}

#[async_trait]
pub trait RegisterParamsExtractor: Send + Sync + 'static {
    async fn client_id(&self) -> Result<Uuid, RegisterParamsExtractorError>;
    async fn user_identifier(&self) -> Result<String, RegisterParamsExtractorError>;
    async fn params(&self, authority: &Authority) -> Result<Value, RegisterParamsExtractorError>;
}

#[derive(Debug)]
pub struct RegisterParamsExtractorError {}

#[async_trait]
pub trait AuthenticateService: Send + Sync + 'static {
    async fn authenticate(&self) -> Result<(), AuthenticateError>;
}

#[async_trait]
pub trait AuthenticateParamsExtractor: Send + Sync + 'static {
    async fn client_id(&self) -> Result<Uuid, AuthenticateParamsExtractorError>;
    async fn user_identifier(&self) -> Result<String, AuthenticateParamsExtractorError>;
    async fn params(
        &self,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<Value, AuthenticateParamsExtractorError>;
}

#[derive(Debug)]
pub struct AuthenticateParamsExtractorError {}

#[derive(Debug)]
pub struct AuthenticateError {}

#[derive(Debug)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: Value,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AuthorityStatus {
    Enabled,
    Disabled,
}

#[derive(Debug)]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
}

#[derive(Debug)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
