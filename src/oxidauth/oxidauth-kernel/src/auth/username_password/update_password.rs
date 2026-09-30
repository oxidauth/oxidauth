use std::{fmt, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

#[derive(Serialize, Deserialize)]
pub struct UpdatePasswordParams {
    pub code: String,
    pub username: String,
    pub client_key: Uuid,
    pub password: String,
    pub password_conf: String,
}

impl fmt::Debug for UpdatePasswordParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `Debug` is what every `#[tracing::instrument]` layer on this DTO
        // records (api handler, service use case, SDK wrapper), so the
        // secret-bearing fields stay constant-masked (OXA-000008):
        // no length, no prefix may leak. The fields stay plain `String` and
        // `Serialize` stays raw — the SDK posts this struct as the request
        // body, so only `Debug` may redact.
        f.debug_struct("UpdatePasswordParams")
            .field("code", &"******")
            .field("username", &self.username)
            .field("client_key", &self.client_key)
            .field("password", &"******")
            .field("password_conf", &"******")
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordResponse {
    pub success: bool,
}

#[async_trait]
pub trait UpdatePasswordServiceTrait: Send + Sync + 'static {
    async fn update_password(
        &self,
        params: &UpdatePasswordParams,
    ) -> Result<UpdatePasswordResponse, BoxedError>;
}

pub type UpdatePasswordService = Arc<dyn UpdatePasswordServiceTrait>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const SENTINEL_CODE: &str = "SENTINEL-recovery-code";
    const SENTINEL_NEW_PW: &str = "SENTINEL-brand-new-password";

    fn params() -> UpdatePasswordParams {
        UpdatePasswordParams {
            code: SENTINEL_CODE.to_owned(),
            username: "malreynolds".to_owned(),
            client_key: Uuid::nil(),
            password: SENTINEL_NEW_PW.to_owned(),
            password_conf: SENTINEL_NEW_PW.to_owned(),
        }
    }

    #[test]
    fn debug_masks_the_passwords_and_the_recovery_code() {
        // OXA-000008: the three instrument layers record this argument
        // through `Debug`; every secret field is the constant mask.
        let debug = format!("{:?}", params());

        assert_eq!(
            debug,
            "UpdatePasswordParams { code: \"******\", username: \"malreynolds\", \
             client_key: 00000000-0000-0000-0000-000000000000, \
             password: \"******\", password_conf: \"******\" }"
        );
        assert!(!debug.contains(SENTINEL_NEW_PW));
        assert!(!debug.contains(SENTINEL_CODE));
    }

    #[test]
    fn serialization_keeps_the_wire_contract_raw() {
        // the SDK posts this struct as the request body: `Serialize` must
        // stay raw, this is the one DTO where the raw pass-through is the
        // deliberate contract (contrast `Password`).
        assert_eq!(
            serde_json::to_value(params()).unwrap(),
            json!({
                "code": SENTINEL_CODE,
                "username": "malreynolds",
                "client_key": Uuid::nil(),
                "password": SENTINEL_NEW_PW,
                "password_conf": SENTINEL_NEW_PW,
            })
        );
    }
}
