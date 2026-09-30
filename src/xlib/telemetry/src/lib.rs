//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
use std::{
    collections::HashMap,
    env::{VarError, var},
    error::Error,
    fmt::Display,
};

use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

const ENVIRONMENT: &str = "ENVIRONMENT";
const RUST_LOG: &str = "RUST_LOG";

pub fn get_subscriber<Sink>(
    name: &str,
    version: &str,
    environment: &str,
    env_filter: &str,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Same VarError policy as get_logging_envs (OXA-000046): a corrupt
    // RUST_LOG must not be masked at subscriber build — the independent
    // EnvFilter read swallows it in an opaque FromEnvError, so the kind is
    // checked on an explicit read.
    let env_filter = match var(RUST_LOG) {
        // Present: strict env parse; invalid directives still fall back to
        // the caller's filter (pre-existing behavior, unchanged).
        Ok(_) => EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter)),
        // Missing: the supported default path — use the caller's filter
        // (binaries pass get_logging_envs' resolved level here).
        Err(VarError::NotPresent) => EnvFilter::new(env_filter),
        // Corrupt value (NotUnicode): fail fast with parity to the boot
        // abort; get_subscriber has no Result, so panic like init_subscriber.
        Err(err) => panic!("{}", EnvVarError::RustLog(err)),
    };

    let default_fields = {
        let mut f = HashMap::new();

        f.insert("application_name".into(), name.into());
        f.insert("application_version".into(), version.into());
        f.insert("application_environment".into(), environment.into());

        f
    };

    let formatting_layer =
        BunyanFormattingLayer::with_default_fields(name.to_owned(), sink, default_fields);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn get_logging_envs() -> Result<(String, String), EnvVarError> {
    let environment = var(ENVIRONMENT).map_err(EnvVarError::Environment)?;

    let tracing_level = match var(RUST_LOG) {
        Ok(level) => level,
        // Missing RUST_LOG is the supported default path (compose/helm
        // pre-set it): boot at INFO, but say so on stderr — no subscriber
        // exists yet here, so a tracing event would be silently discarded.
        Err(VarError::NotPresent) => {
            eprintln!("RUST_LOG not set; defaulting to INFO");

            "INFO".into()
        },
        // Corrupt value (NotUnicode): same policy as ENVIRONMENT — fail fast.
        Err(err) => return Err(EnvVarError::RustLog(err)),
    };

    Ok((environment, tracing_level))
}

#[derive(Debug)]
pub enum EnvVarError {
    Environment(VarError),
    RustLog(VarError),
}

impl Display for EnvVarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (vari, err) = match self {
            Self::Environment(err) => (ENVIRONMENT, err),
            Self::RustLog(err) => (RUST_LOG, err),
        };

        write!(f, "env var {vari} not found: {err:?}")
    }
}

impl Error for EnvVarError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Environment(err) => Some(err),
            Self::RustLog(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{LazyLock, Mutex, MutexGuard};

    use super::*;

    /// ENVIRONMENT/RUST_LOG are process-wide state: every env-touching test
    /// MUST hold this guard so cargo's test threads can't interleave.
    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // SAFETY for every set_var/remove_var below: only called while `env_guard()`
    // is held by the current thread, so no other test thread observes a torn env.
    fn set_env(key: &str, value: &str) {
        unsafe { std::env::set_var(key, value) };
    }

    fn remove_env(key: &str) {
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn returns_the_environment_and_rust_log_pair() {
        let _guard = env_guard();
        set_env(ENVIRONMENT, "development");
        set_env(RUST_LOG, "debug,sqlx=warn");

        assert_eq!(
            get_logging_envs().unwrap(),
            ("development".to_string(), "debug,sqlx=warn".to_string())
        );
    }

    #[test]
    fn missing_environment_is_an_environment_variant_of_not_present() {
        let _guard = env_guard();
        remove_env(ENVIRONMENT);
        set_env(RUST_LOG, "info");

        match get_logging_envs() {
            Err(EnvVarError::Environment(VarError::NotPresent)) => {},
            other => panic!("expected Environment(NotPresent), got {other:?}"),
        }
    }

    #[test]
    fn missing_rust_log_falls_back_to_info_instead_of_erroring() {
        // Intentional default (OXA-000046 Option A): a missing RUST_LOG is
        // the supported boot path — fall back to "INFO" and print an
        // eprintln! notice. Only a corrupt (NotUnicode) value surfaces as
        // EnvVarError::RustLog.
        let _guard = env_guard();
        set_env(ENVIRONMENT, "production");
        remove_env(RUST_LOG);

        assert_eq!(
            get_logging_envs().unwrap(),
            ("production".to_string(), "INFO".to_string())
        );
    }

    #[test]
    fn environment_is_resolved_before_rust_log() {
        let _guard = env_guard();
        remove_env(ENVIRONMENT);
        remove_env(RUST_LOG);

        // Both absent: the ENVIRONMENT failure wins — ENVIRONMENT resolves
        // first, and an absent RUST_LOG is the intentional INFO default,
        // not an error, by policy.
        match get_logging_envs() {
            Err(EnvVarError::Environment(VarError::NotPresent)) => {},
            other => panic!("expected Environment(NotPresent), got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_environment_surfaces_as_not_unicode_kind() {
        let _guard = env_guard();
        remove_env(RUST_LOG);
        // Invalid UTF-8 is representable in a unix environment block, so this
        // exercises VarError::NotUnicode rather than NotPresent.
        use std::os::unix::ffi::OsStrExt;

        unsafe { std::env::set_var(ENVIRONMENT, std::ffi::OsStr::from_bytes(&[0xFF, 0xFE])) };

        match get_logging_envs() {
            Err(EnvVarError::Environment(VarError::NotUnicode(_))) => {},
            other => panic!("expected Environment(NotUnicode), got {other:?}"),
        }

        remove_env(ENVIRONMENT);
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_rust_log_is_returned_as_rust_log_variant() {
        let _guard = env_guard();
        set_env(ENVIRONMENT, "staging");
        // Invalid UTF-8 is representable in a unix environment block.
        use std::os::unix::ffi::OsStrExt;

        unsafe { std::env::set_var(RUST_LOG, std::ffi::OsStr::from_bytes(&[0xFF, 0xFE])) };

        match get_logging_envs() {
            Err(EnvVarError::RustLog(VarError::NotUnicode(_))) => {},
            other => panic!("expected RustLog(NotUnicode), got {other:?}"),
        }

        remove_env(RUST_LOG);
    }

    #[cfg(unix)]
    #[test]
    fn get_subscriber_panics_on_non_unicode_rust_log() {
        // Companion pin (OXA-000046): the subscriber build must not mask a
        // corrupt RUST_LOG — it fails fast with the boot-abort message.
        let _guard = env_guard();
        use std::os::unix::ffi::OsStrExt;

        unsafe { std::env::set_var(RUST_LOG, std::ffi::OsStr::from_bytes(&[0xFF, 0xFE])) };

        let outcome = std::panic::catch_unwind(|| {
            get_subscriber("telemetry-tests", "0.0.0", "test", "INFO", std::io::stdout);
        });

        remove_env(RUST_LOG);

        let err = outcome.expect_err("corrupt RUST_LOG must abort the subscriber build");
        let msg = err
            .downcast_ref::<String>()
            .expect("panic payload is the formatted message");

        assert!(
            msg.starts_with("env var RUST_LOG not found: NotUnicode"),
            "unexpected panic message: {msg}"
        );
    }

    #[test]
    fn display_names_the_var_and_inner_kind_for_both_variants() {
        assert_eq!(
            EnvVarError::Environment(VarError::NotPresent).to_string(),
            "env var ENVIRONMENT not found: NotPresent"
        );
        assert_eq!(
            EnvVarError::RustLog(VarError::NotPresent).to_string(),
            "env var RUST_LOG not found: NotPresent"
        );
    }

    #[test]
    fn source_returns_the_wrapped_var_error() {
        for err in [
            EnvVarError::Environment(VarError::NotPresent),
            EnvVarError::RustLog(VarError::NotPresent),
        ] {
            assert!(
                matches!(
                    err.source()
                        .and_then(|src| src.downcast_ref::<VarError>()),
                    Some(VarError::NotPresent)
                ),
                "source() must expose the inner VarError for {err:?}"
            );
        }
    }
}
