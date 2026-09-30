use oxidauth_kernel::{
    auth::register::RegisterParams,
    invitations::accept_invitation::AcceptInvitationUserParams,
    users::User,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationPathReq {
    pub invitation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationBodyReq {
    pub user: AcceptInvitationUserParams,
    pub user_authority: RegisterParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationRes {
    pub user: User,
}
