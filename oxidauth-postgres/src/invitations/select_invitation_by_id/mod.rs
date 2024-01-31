use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{find_invitation::FindInvitationParams, Invitation},
    service::Service,
};

use crate::Database;

use super::PgInvitation;

#[async_trait]
impl<'a> Service<&'a FindInvitationParams> for Database {
    type Response = Invitation;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_invitation_by_id_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgInvitation>(include_str!(
            "./select_invitation_by_id_query.sql"
        ))
        .bind(params.id)
        .fetch_one(&self.pool)
        .await?;

        let invitation = result.into();

        Ok(invitation)
    }
}
