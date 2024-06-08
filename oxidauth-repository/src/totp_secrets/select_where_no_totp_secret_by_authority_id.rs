pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub struct SelectWhereNoTotpSecretByAuthorityIdParams {
    pub authority_id: Uuid,
}

#[async_trait]
pub trait SelectWhereNoTotpSecretByAuthorityIdQuery:
    Send + Sync + 'static
{
    async fn select_where_no_totp_secret_by_authority_id(
        &self,
        authority_id: Uuid,
    ) -> Result<Vec<Uuid>, BoxedError>;
}
