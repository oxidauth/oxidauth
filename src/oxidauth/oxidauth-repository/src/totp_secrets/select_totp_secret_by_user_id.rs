use oxidauth_kernel::totp_secrets::{
    TOTPSecret,
    find_totp_secret_by_user_id::FindTOTPSecretByUserId,
};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectTOTPSecrețByUserIdQuery: Send + Sync + 'static {
    async fn select_totp_secret_by_user_id(
        &self,
        params: &FindTOTPSecretByUserId,
    ) -> Result<TOTPSecret, BoxedError>;
}
