use oxidauth_kernel::invitations::{Invitation, find_invitation::FindInvitationParams};
use serde::{Deserialize, Serialize};

pub type FindInvitationReq = FindInvitationParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindInvitationRes {
    pub invitation: Invitation,
}
