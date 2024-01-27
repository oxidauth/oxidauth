use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{delete_invitation::DeleteInvitationParams, Invitation},
    service::Service,
};

use crate::Database;

use super::PgInvitation;

#[async_trait]
impl<'a> Service<&'a DeleteInvitationParams> for Database {
    type Response = Invitation;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a DeleteInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgInvitation>(include_str!(
            "./delete_invitation_by_id_query.sql"
        ))
        .bind(params.id)
        .fetch_one(&self.pool)
        .await?;

        let invitation = result.into();

        Ok(invitation)
    }
}
