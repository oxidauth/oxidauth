use crate::dev_prelude::*;

use super::User;

#[async_trait]
pub trait FindUserByIdService: Send + Sync + 'static {
    async fn find_user_by_id(
        &self,
        user_id: Uuid,
    ) -> Result<User, FindUserByIdError>;
}

#[derive(Debug)]
pub struct FindUserByIdError {}
