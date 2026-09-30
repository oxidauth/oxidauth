use oxidauth_kernel::invitations::create_invitation::{
    CreateInvitationParams,
    CreateInvitationResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitationReq {
    pub invitation: CreateInvitationParams,
}

pub type CreateInvitationRes = CreateInvitationResponse;
