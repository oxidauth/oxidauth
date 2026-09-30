use async_trait::async_trait;
use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    auth::username_password::forgot_password::{
        ForgotPasswordParams,
        ForgotPasswordResponse,
        ForgotPasswordServiceTrait,
    },
    error::BoxedError,
    refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserId,
    totp_secrets::find_totp_secret_by_user_id::FindTOTPSecretByUserId,
};
use oxidauth_repository::{
    refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserIdQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
};

use crate::dev_prelude::epoch;

pub struct ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    delete_refresh_tokens_by_user_id: D,
    user_totp_secret: S,
}

impl<D, S> ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    pub fn new(delete_refresh_tokens_by_user_id: D, user_totp_secret: S) -> Self {
        Self {
            delete_refresh_tokens_by_user_id,
            user_totp_secret,
        }
    }
}

#[async_trait]
impl<D, S> ForgotPasswordServiceTrait for ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    #[tracing::instrument(name = "ForgotPasswordUseCase::forgot_password", skip(self))]
    async fn forgot_password(
        &self,
        params: &ForgotPasswordParams,
    ) -> Result<ForgotPasswordResponse, BoxedError> {
        // generate code
        let secret_by_user_id = self
            .user_totp_secret
            .select_totp_secret_by_user_id(&FindTOTPSecretByUserId {
                user_id: params.user_id,
            })
            .await?;

        let now = epoch()?;

        let code = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(600)
            .timestamp(now)
            .finalize()
            .map_err(|err| format!("error generating totp: {:?}", err))?
            .generate();

        // delete refresh tokens by id
        self.delete_refresh_tokens_by_user_id
            .delete_refresh_token_by_user_id(&DeleteRefreshTokenByUserId {
                user_id: params.user_id,
            })
            .await?;

        Ok(ForgotPasswordResponse { code })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use chrono::Utc;
    use oxidauth_kernel::{refresh_tokens::RefreshToken, totp_secrets::TOTPSecret};
    use uuid::Uuid;

    use super::*;
    use crate::auth::strategies::username_password::fixtures::USER_ID;

    const TOTP_SECRET: &str = "12345678901234567890";

    #[derive(Clone, Default)]
    struct CallLog {
        totp_requests: Arc<Mutex<Vec<Uuid>>>,
        delete_requests: Arc<Mutex<Vec<Uuid>>>,
    }

    impl CallLog {
        fn totp_requests(&self) -> Vec<Uuid> {
            self.totp_requests
                .lock()
                .expect("log")
                .clone()
        }

        fn delete_requests(&self) -> Vec<Uuid> {
            self.delete_requests
                .lock()
                .expect("log")
                .clone()
        }
    }

    struct MockTotpSecret {
        secret: String,
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectTOTPSecrețByUserIdQuery for MockTotpSecret {
        async fn select_totp_secret_by_user_id(
            &self,
            req: &FindTOTPSecretByUserId,
        ) -> Result<TOTPSecret, BoxedError> {
            self.log
                .totp_requests
                .lock()
                .expect("log")
                .push(req.user_id);

            if self.fail {
                return Err("simulated totp secret failure".into());
            }

            Ok(TOTPSecret {
                secret: self.secret.clone(),
            })
        }
    }

    struct MockDeleteRefreshTokens {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl DeleteRefreshTokenByUserIdQuery for MockDeleteRefreshTokens {
        async fn delete_refresh_token_by_user_id(
            &self,
            req: &DeleteRefreshTokenByUserId,
        ) -> Result<Vec<RefreshToken>, BoxedError> {
            self.log
                .delete_requests
                .lock()
                .expect("log")
                .push(req.user_id);

            if self.fail {
                return Err("simulated delete failure".into());
            }

            // one-element vec keeps the mock representative of a bulk revoke
            Ok(vec![RefreshToken {
                id: Uuid::new_v4(),
                user_id: req.user_id,
                authority_id: Uuid::new_v4(),
                expires_at: Utc::now(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        }
    }

    fn use_case(
        log: &CallLog,
        totp_fails: bool,
        delete_fails: bool,
    ) -> ForgotPasswordUseCase<MockDeleteRefreshTokens, MockTotpSecret> {
        ForgotPasswordUseCase::new(
            MockDeleteRefreshTokens {
                fail: delete_fails,
                log: log.clone(),
            },
            MockTotpSecret {
                secret: TOTP_SECRET.to_owned(),
                fail: totp_fails,
                log: log.clone(),
            },
        )
    }

    fn expected_code() -> String {
        // mirrors the use-case builder: ascii key, 600s period, current epoch
        TOTPBuilder::new()
            .ascii_key(TOTP_SECRET)
            .period(600)
            .timestamp(epoch().expect("epoch"))
            .finalize()
            .expect("totp config is valid")
            .generate()
    }

    async fn stay_inside_totp_window() {
        // zero-tolerance `is_valid` semantics: pin both sides into the same
        // 600s bucket so the counters cannot straddle a boundary
        while epoch()
            .expect("epoch")
            .rem_euclid(600)
            > 595
        {
            tokio::time::sleep(Duration::from_millis(1_100)).await;
        }
    }

    #[tokio::test]
    async fn returns_a_fresh_totp_code_and_revokes_the_users_refresh_tokens() {
        // Pinned: `forgot_password` itself performs no identity verification —
        // the real auth gate lives in the route: the axum handler at
        // `oxidauth-api`'s v1/auth/username_password/forgot_password.rs
        // requires `ExtractJwt` + `ExtractEntitlements` and answers the
        // `oxidauth:auth:forgot_password` challenge (`PERMISSION`) before the
        // service is fetched (OXA-000005 Step 1).
        stay_inside_totp_window().await;

        let log = CallLog::default();
        let case = use_case(&log, false, false);

        let res = case
            .forgot_password(&ForgotPasswordParams { user_id: USER_ID })
            .await
            .expect("happy path must succeed");

        assert_eq!(
            res.code,
            expected_code(),
            "the returned code must be the current window TOTP of the user's secret"
        );
        assert_eq!(
            log.totp_requests().as_slice(),
            &[USER_ID],
            "the secret must be fetched for the requested user"
        );
        assert_eq!(
            log.delete_requests()
                .as_slice(),
            &[USER_ID],
            "forgot-password must revoke the user's refresh tokens exactly once"
        );
    }

    #[tokio::test]
    async fn totp_secret_lookup_failure_propagates_without_deleting_tokens() {
        let log = CallLog::default();
        let case = use_case(&log, true, false);

        let err = case
            .forgot_password(&ForgotPasswordParams { user_id: USER_ID })
            .await
            .expect_err("secret lookup failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated totp secret failure")
        );
        assert!(
            log.delete_requests()
                .is_empty(),
            "tokens must not be deleted when the code cannot be generated"
        );
    }

    #[tokio::test]
    async fn refresh_token_delete_failure_propagates() {
        let log = CallLog::default();
        let case = use_case(&log, false, true);

        let err = case
            .forgot_password(&ForgotPasswordParams { user_id: USER_ID })
            .await
            .expect_err("delete failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated delete failure")
        );
        assert_eq!(log.totp_requests().as_slice(), &[USER_ID]);
    }
}
