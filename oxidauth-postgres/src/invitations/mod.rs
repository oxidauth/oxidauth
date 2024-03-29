use chrono::{DateTime, Utc};
use oxidauth_kernel::invitations::Invitation;
use uuid::Uuid;

pub mod delete_invitation_by_id;
pub mod insert_invitation;
pub mod select_invitation_by_id;

#[derive(Debug, sqlx::FromRow)]
pub struct PgInvitation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgInvitation> for Invitation {
    fn from(value: PgInvitation) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
