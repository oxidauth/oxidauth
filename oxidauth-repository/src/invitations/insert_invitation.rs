use async_trait::async_trait;
use chrono::{DateTime, Utc};
use oxidauth_kernel::{
    error::BoxedError, invitations::Invitation, service::Service,
};
use uuid::Uuid;

#[async_trait]
pub trait InsertInvitationQuery:
    for<'a> Service<
    &'a InsertInvitationParams,
    Response = Invitation,
    Error = BoxedError,
>
{
}

impl<T> InsertInvitationQuery for T where
    T: for<'a> Service<
        &'a InsertInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct InsertInvitationParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}
