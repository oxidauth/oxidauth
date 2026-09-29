//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
use std::{
    collections::HashMap,
    env::{VarError, var},
    error::Error,
    fmt::Display,
};

use tracing::{Subscriber, error, info, subscriber::set_global_default};
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
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));

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

    let tracing_level = var(RUST_LOG).unwrap_or_else(|err| {
        error!(
            "no valid tracing level provided: {:?}",
            EnvVarError::RustLog(err)
        );

        info!("falling back to INFO");

        "INFO".into()
    });

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