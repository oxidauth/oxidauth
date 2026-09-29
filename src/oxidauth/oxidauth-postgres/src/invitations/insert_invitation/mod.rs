use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError, invitations::Invitation, service::Service,
};
use oxidauth_repository::invitations::insert_invitation::InsertInvitationParams;

use crate::Database;

use super::PgInvitation;

#[async_trait]
impl<'a> Service<&'a InsertInvitationParams> for Database {
    type Response = Invitation;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_invitation_query", skip(self))]
    async fn call(
        &self,
        params: &'a InsertInvitationParams,
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
