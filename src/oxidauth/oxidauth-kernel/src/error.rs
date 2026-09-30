use std::{error::Error, fmt::Debug};

use serde::Serialize;

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

/// The wire error envelope: serialized verbatim as one value of the `errors`
/// array of the API `Response` JSON.
#[derive(Debug, Serialize)]
pub struct OxidAuthError<C, E>
where
    C: Debug + Serialize,
    E: Debug + Serialize,
{
    pub name: String,
    /// The error's top-level `Display` — human-readable prose.
    pub display: String,
    /// The error's top-level `Debug`; transitively includes the cause's `Debug`.
    /// Machine-shaped text — the hurl suite asserts against this field.
    pub debug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<C>,
    /// The `Display` of the immediate `std::error::Error::source()` cause —
    /// `Display`, not `Debug` (intended contract, OXA-000047). The key is omitted
    /// from the serialized JSON when the error has no cause (`skip_serializing_if`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<E>,
}

/// Renders an error into the [`OxidAuthError`] wire envelope.
pub trait IntoOxidAuthError<C, E>
where
    C: Debug + Serialize,
    E: Debug + Serialize,
{
    /// Fills the envelope per its wire contract: `display` = top-level `Display`,
    /// `debug` = top-level `Debug` (transitively includes the cause's `Debug`),
    /// `source` = `Display` of the immediate `std::error::Error::source()` cause,
    /// omitted from the JSON when absent.
    fn into_error(self) -> OxidAuthError<C, E>;
}

impl IntoOxidAuthError<(), String> for Box<dyn Error + Send + Sync + 'static> {
    fn into_error(self) -> OxidAuthError<(), String> {
        OxidAuthError {
            name: "BoxedError".to_owned(),
            display: format!("{}", self),
            debug: format!("{:?}", self),
            status_code: None,
            context: None,
            source: self
                .source()
                .map(|s| s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use serde_json::json;

    use super::*;

    // stands in for `sqlx::Error::RowNotFound`, whose derived Debug string
    // ("RowNotFound") is exactly what the hurl suite asserts against via
    // `$.errors[0].debug`
    #[derive(Debug)]
    struct RowNotFound;

    impl fmt::Display for RowNotFound {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "database row not found")
        }
    }

    impl Error for RowNotFound {
    }

    #[derive(Debug)]
    struct Wrapped(RowNotFound);

    impl fmt::Display for Wrapped {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "auth lookup failed")
        }
    }

    impl Error for Wrapped {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn into_error_envelopes_a_sourceless_error() {
        let boxed: BoxedError = Box::new(RowNotFound);

        let err = boxed.into_error();

        assert_eq!(err.name, "BoxedError");
        assert_eq!(err.display, "database row not found");

        // the Debug string passes through verbatim — hurl asserts contain
        // "RowNotFound" on this exact field
        assert_eq!(err.debug, "RowNotFound");

        assert_eq!(err.status_code, None);
        assert_eq!(err.context, None);
        assert_eq!(err.source, None);
    }

    #[test]
    fn into_error_captures_the_error_source_as_display() {
        let boxed: BoxedError = Box::new(Wrapped(RowNotFound));

        let err = boxed.into_error();

        assert_eq!(err.display, "auth lookup failed");

        // Contract (OXA-000047), pinned below: `source` is the *Display* of the
        // immediate `Error::source()` cause, not its Debug — consumers see the
        // human-readable cause. Do NOT flip to Debug: the top-level `debug`
        // field already embeds the nested Debug, so Debug-ing `source` would
        // duplicate that text under two keys and change the wire JSON for
        // every PgError-wrapped failure.
        assert_eq!(err.source, Some("database row not found".to_string()));

        // `debug` is the *top-level* Debug, which includes the nested Debug
        assert_eq!(err.debug, "Wrapped(RowNotFound)");
    }

    #[test]
    fn oxid_auth_error_json_skips_absent_optionals() {
        let boxed: BoxedError = Box::new(RowNotFound);

        let json = serde_json::to_value(boxed.into_error()).unwrap();

        // exactly the three always-present keys; every hurl assert on
        // `$.errors[0].*` and `$.payload not exists` assumes this key set
        assert_eq!(
            json,
            json!({
                "name": "BoxedError",
                "display": "database row not found",
                "debug": "RowNotFound",
            })
        );
    }

    #[test]
    fn oxid_auth_error_json_includes_present_source() {
        let boxed: BoxedError = Box::new(Wrapped(RowNotFound));

        let json = serde_json::to_value(boxed.into_error()).unwrap();

        assert_eq!(json["source"], json!("database row not found"));
        assert_eq!(json["debug"], json!("Wrapped(RowNotFound)"));
        assert!(
            json.get("status_code")
                .is_none()
        );
        assert!(json.get("context").is_none());
    }
}
