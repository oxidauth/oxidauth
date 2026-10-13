pub mod client_secret;
pub mod exchange_token;
pub mod id_token;

use std::{error::Error, fmt};

pub use client_secret::build_client_secret;
pub use exchange_token::exchange_apple_token;
pub use id_token::retrieve_apple_profile;
use serde::{Deserialize, Serialize};

/// Apple's issuer and audience value — the same string serves as the JWT
/// `aud` for client secrets and the `iss` of id_tokens (OXA-000072).
pub const APPLE_ISSUER: &str = "https://appleid.apple.com";

#[derive(Debug, Clone, Serialize)]
pub struct AppleExchangeTokenReq<'a> {
    pub code: &'a str,
    pub client_id: &'a str,     // Services ID (reuses `oauth2_id`)
    pub client_secret: &'a str, // the ES256 client-secret JWT, NOT a static secret
    pub redirect_uri: &'a str,
    pub grant_type: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleExchangeTokenRes {
    /// The only field this flow cannot do without — identity lives inside it.
    pub id_token: String,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// Domain errors for the Sign in with Apple exchange and verification paths
/// (OXA-000072). Messages are deliberately sanitized: no key material, no
/// claim contents, and no raw IdP bodies ever reach the caller — unlike the
/// `map_err(err.to_string())` transport-prose contract the Google/Microsoft
/// paths are pinned to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppleAuthError {
    /// An `apple_*` authority param is absent on an Apple flavor.
    MissingParam(&'static str),
    /// The named env var is not set in this process.
    MissingEnvVar(String),
    /// The PEM behind the env var name is not a usable PKCS#8 EC key.
    InvalidPrivateKey,
    /// Signing the client-secret JWT failed.
    Signing,
    /// Network/transport failure talking to Apple.
    Transport,
    /// Apple answered non-2xx; `error` carries the OAuth `error` code only.
    IdpRejected { status: u16, error: Option<String> },
    /// A 2xx token response that still lacks the id_token we live by.
    MissingIdToken,
    /// The id_token failed header/JWKS lookup or signature/claim validation.
    TokenRejected(String),
    /// The id_token nonce does not match sha256 of the consumed state nonce.
    NonceMismatch,
    /// The id_token email is absent, or Apple flagged it unverified.
    UnverifiedEmail,
}

impl fmt::Display for AppleAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AppleAuthError::*;

        match self {
            MissingParam(name) => write!(f, "apple authority params missing `{name}`"),
            MissingEnvVar(name) => write!(f, "apple private key env var `{name}` is not set"),
            InvalidPrivateKey => write!(f, "apple private key is not a valid PKCS#8 EC PEM"),
            Signing => write!(f, "could not sign the apple client secret"),
            Transport => write!(f, "apple request transport failed"),
            IdpRejected { status, error } => {
                match error {
                    Some(error) => write!(f, "apple rejected the request ({status}): {error}"),
                    None => write!(f, "apple rejected the request ({status})"),
                }
            },
            MissingIdToken => write!(f, "apple token response carried no id_token"),
            TokenRejected(reason) => write!(f, "apple id_token rejected: {reason}"),
            NonceMismatch => write!(f, "apple id_token nonce does not match the state"),
            UnverifiedEmail => write!(f, "apple id_token carried no verified email"),
        }
    }
}

impl Error for AppleAuthError {
}

/// Test-only Apple key fixtures shared across the apple modules and the
/// authenticate-or-register end-to-end flow (OXA-000072). These keys are
/// throwaway and must never be trusted for anything, anywhere.
#[cfg(test)]
pub(crate) mod fixtures {
    use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use sha2::{Digest, Sha256};

    pub(crate) const EC_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
        MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgnDf57ZQDj+IP2Drl\n\
        tW7BmLpLKg1DN2PDZS33HJFQeNChRANCAAS4/Mdlkbcs5N+aIc62qj600bsRro94\n\
        DOJfZFraqxxFU6XndnC6rq5m5Y7oid0C3mSTvoXIH//qnjVxtI4ad11t\n\
        -----END PRIVATE KEY-----\n";

    pub(crate) const EC_PUB_PEM: &str = "-----BEGIN PUBLIC KEY-----\n\
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEuPzHZZG3LOTfmiHOtqo+tNG7Ea6P\n\
        eAziX2Ra2qscRVOl53Zwuq6uZuWO6IndAt5kk76FyB//6p41cbSOGnddbQ==\n\
        -----END PUBLIC KEY-----\n";

    pub(crate) const RSA_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
        MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDVh8WSSE6dnJ/e\n\
        4kwC4WKvZnz+h6OerIcGpEhZyjVFX+3CI10kBVaHY2AHOwVztwBPCuNWWWIMd4cj\n\
        xDtpnW8Wa/LYr5wjWspWk84S1Mq2BQBQugc19pBCCFLIdvq1QNTihkfDsvCnRC9q\n\
        pEBKZZv9E/ajIFxHrJlFhv2TN01INMuMuPhvosCEuKG26BDN5eu4R0Eqpk4hU2hz\n\
        /ZFBAOPgk9zGDLtPn8J0JhBtkX2WxW54pvVBzPoGRyKq2507RGfUwAeJyxShhojM\n\
        9HtP9Iq/vN3wuFACq6elSSGV/WuiF7mCWbx0lemz96CnSjJtqvGMw6EoJxkjhLBd\n\
        8WuaWufdAgMBAAECggEAOurajfFbl8uWpSYluQm9dA5AYO7GK8sI+sEroXLU3g6J\n\
        CO8/DOhYJsU5wuT7LSGKeUTU04eYMT5t0GWdyxeAhlxExps15t9vQxJHYxTJ1g1t\n\
        N9kMkkw1I7MuPFfv90ErPFG2zX27PzvmJXOqtZxxvRPcEsikDbbHF9MWdp9kuI4v\n\
        KujxBn00cb5kvns54T0xgv5p/xUjtIJsInacsUxhTMXswMBfNhvavTlv9HLfdVeE\n\
        DWKiCMRUIzr4BDahGw5PsWDv7Yd4Eyn4GNvKsDCmKw2mX/QpmuObJIvUbS6shUJI\n\
        6cecWbe3hhOmc7RXp4HwbgfwaBFG9/ko1sK8ChLkVQKBgQD/6AcNlUNtYkuJLAOz\n\
        dDlBF60yaqJAH0osN3B7WrqcEPyxj2TqHqQSMg84RRBtSA/C/Tj8TSkIAXoUTNbp\n\
        gTLLZ4l9Jgn/Pe5PuAWES2sgnPKJpF20evyoeYeTrMT3PsN/mjANtJiT1FQhAiFL\n\
        zcs31OgSuQ0SJCCt8j4y3nbmKwKBgQDVm8ZKR2RH1oTYJPZlPkeeJwXkJRDwx2HO\n\
        YVk9mMrk78e73AyhF+kIJLJpmBd0R99JIqCl/YL9R57JqTmdvtyyKXLxpNBdT773\n\
        gl5CUZo2vHsO+jGPA7lGNVOZBifozS7n85De6EBAfEEkkYqJRpeDBCTpVevDDg4x\n\
        VoNfvTGuFwKBgQCfe2/h1DMYWVjv3OFVlkoUOfgzYmdJhPaJxecwySNhHekNk2r1\n\
        yTDBMjdv/wK0yGMiRwOboU6/8YOuYRDSpzULtLPnrawvSN5bCjlN8F6MDaPNigTn\n\
        9Hm/WTXYH6UE3TWjMWlfMTyhbIMdqyErB87tNZ+IA5GUy7UeEskWlLQOzwKBgE6W\n\
        8W2CtrJ3ioi7kKvPLipR0SkPOhxcPu/fgSXYgU2ZqrLDUyIx2drVJY8UFg0CJHST\n\
        +OcZTohBQXqQeFJBVpF2DoQ5TzAFlw0ZOfTMcSCSV/Q6uOEVTaU3GXrDeQt98jQE\n\
        uNhKOtl+3bjGk2HORxA9BheTNRqeoTolyk//G7ghAoGALhP7o5fgczusxckf2Qf7\n\
        ANCpaX2rQW7lWw1M93WSe49dbT+A3A3kWqLQ/7xC/RgpIXe4tS9EUOF5bMNLi8aA\n\
        Lnv4cOAugpnWWWcwDz/GOzokaLTtesF5xpeHCv9KLgHb8R4WNaxROcNtsb8tSf/r\n\
        lsLsgc5vmf5l89PsLTevbkk=\n\
        -----END PRIVATE KEY-----\n";

    // RFC 7518 base64url components of the fixture key's modulus/exponent —
    // what Apple's JWKS carries for this kid.
    pub(crate) const RSA_N: &str = "1YfFkkhOnZyf3uJMAuFir2Z8_oejnqyHBqRIWco1RV_twiNdJAVWh2NgBzsFc7cATwrjVlliDHeHI8Q7aZ1vFmvy2K-cI1rKVpPOEtTKtgUAULoHNfaQQghSyHb6tUDU4oZHw7Lwp0QvaqRASmWb_RP2oyBcR6yZRYb9kzdNSDTLjLj4b6LAhLihtugQzeXruEdBKqZOIVNoc_2RQQDj4JPcxgy7T5_CdCYQbZF9lsVueKb1Qcz6BkciqtudO0Rn1MAHicsUoYaIzPR7T_SKv7zd8LhQAqunpUkhlf1rohe5glm8dJXps_egp0oybarxjMOhKCcZI4SwXfFrmlrn3Q";
    pub(crate) const RSA_E: &str = "AQAB";
    pub(crate) const KID: &str = "AIDXbigxKz";

    #[derive(Debug, Clone, serde::Serialize)]
    pub(crate) struct IdTokenClaims {
        pub(crate) iss: String,
        pub(crate) aud: String,
        pub(crate) exp: u64,
        pub(crate) iat: u64,
        pub(crate) sub: String,
        pub(crate) email: Option<String>,
        pub(crate) nonce: Option<String>,
        pub(crate) email_verified: Option<serde_json::Value>,
    }

    pub(crate) fn nonce_for(state: &str) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(state.as_bytes()))
    }

    pub(crate) fn sign_id_token(claims: IdTokenClaims) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.to_owned());

        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(RSA_PEM.as_bytes()).expect("fixture key"),
        )
        .expect("fixture token signs")
    }

    pub(crate) fn jwks_json() -> serde_json::Value {
        serde_json::json!({
            "keys": [{"kty": "RSA", "kid": KID, "use": "sig", "alg": "RS256",
                      "n": RSA_N, "e": RSA_E}],
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::auth::strategies::oauth2::fixtures::apple_authority_params;

    #[test]
    fn exchange_token_request_wire_contract() {
        // mirrors the field mapping inside `exchange_apple_token`: the form
        // body posted to `exchange_url` must carry exactly these wire names
        let params = apple_authority_params();
        let req = AppleExchangeTokenReq {
            code: "one-time-code",
            client_id: &params.oauth2_id,
            client_secret: "signed.jwt.string",
            redirect_uri: params.redirect_uri.as_ref(),
            grant_type: "authorization_code",
        };

        let wire: Value = serde_json::to_value(&req).expect("serializable");

        assert_eq!(
            wire,
            json!({
                "code": "one-time-code",
                "client_id": "sso-client-id",
                "client_secret": "signed.jwt.string",
                "redirect_uri": "https://app.example.com/cb",
                "grant_type": "authorization_code",
            }),
        );
    }

    #[test]
    fn token_response_requires_only_the_id_token() {
        // Apple sends access_token/token_type/expires_in in practice, but the
        // flow cannot function without the id_token and nothing else — so
        // that is the one required field (a Google-style pin that demands
        // every field would turn optional Apple fields into exchange errors).
        let res: AppleExchangeTokenRes = serde_json::from_value(json!({
            "id_token": "eyJhbGciOi.jwt",
            "access_token": "at",
            "token_type": "Bearer",
            "expires_in": 3600,
        }))
        .expect("a full apple token response parses");
        assert_eq!(res.id_token, "eyJhbGciOi.jwt");

        let minimal: AppleExchangeTokenRes = serde_json::from_value(json!({
            "id_token": "eyJhbGciOi.jwt",
        }))
        .expect("an id_token-only response parses");
        assert!(minimal.access_token.is_none());

        // an OAuth error response is a parse failure, mapped by the exchange
        // to a typed IdpRejected — it never arrives as transport prose here
        assert!(
            serde_json::from_value::<AppleExchangeTokenRes>(json!({
                "error": "invalid_grant",
                "error_description": "invalid code",
            }))
            .is_err()
        );
    }

    #[test]
    fn error_display_never_leaks_secrets_or_claims() {
        // sanitized-by-construction: the display strings carry param and env
        // var NAMES, statuses, and OAuth error codes — never key material
        let cases: Vec<(AppleAuthError, Vec<&str>)> = vec![
            (
                AppleAuthError::MissingEnvVar("MY_KEY".to_owned()),
                vec!["MY_KEY"],
            ),
            (
                AppleAuthError::IdpRejected {
                    status: 400,
                    error: Some("invalid_grant".to_owned()),
                },
                vec!["400", "invalid_grant"],
            ),
            (AppleAuthError::NonceMismatch, vec!["nonce"]),
        ];

        for (err, expects) in cases {
            let text = err.to_string();
            for expect in expects {
                assert!(text.contains(expect), "`{text}` should mention `{expect}`");
            }
            assert!(
                !text.contains("PRIVATE KEY") && !text.contains("BEGIN"),
                "error display must never carry key material: {text}"
            );
        }
    }
}
