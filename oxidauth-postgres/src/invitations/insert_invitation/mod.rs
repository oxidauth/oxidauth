use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{create_invitation::CreateInvitationParams, Invitation},
    service::Service,
};

use crate::Database;

use super::PgInvitation;

#[async_trait]
impl<'a> Service<&'a CreateInvitationParams> for Database {
    type Response = Invitation;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a CreateInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgInvitation>(include_str!(
            "./insert_invitation.sql"
        ))
        .bind(params.id)
        .bind(params.user_id)
        .bind(params.expires_at)
        .fetch_one(&self.pool)
        .await?;

        let invitation = result.into();

        Ok(invitation)
    }
}
