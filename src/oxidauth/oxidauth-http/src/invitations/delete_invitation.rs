use oxidauth_kernel::invitations::{Invitation, delete_invitation::DeleteInvitationParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationReq {
    pub invitation: DeleteInvitationParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationRes {
    pub invitation: Invitation,
}
