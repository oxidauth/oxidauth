pub mod accept_invitation;
pub mod create_invitation;
pub mod find_invitation;

pub use oxidauth_kernel::invitations::*;

use super::fmt;
#[cfg(feature = "mock")]
use super::mock::ClientMock;
use crate::{
    Client,
    invitations::{
        accept_invitation::AcceptInvitationTrait,
        create_invitation::CreateInvitationTrait,
        find_invitation::FindInvitationTrait,
    },
};

pub trait InvitationsTrait:
    FindInvitationTrait + CreateInvitationTrait + AcceptInvitationTrait
{
}

impl InvitationsTrait for Client {
}

#[cfg(feature = "mock")]
impl InvitationsTrait for ClientMock {
}
