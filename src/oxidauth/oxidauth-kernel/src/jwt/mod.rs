use std::{
    borrow::Cow,
    fmt,
    io::prelude::*,
    ops::{Add as _, Sub as _},
    str::FromStr,
    time::{self, Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use jsonwebtoken::{
    Algorithm,
    DecodingKey,
    EncodingKey,
    Header,
    TokenData,
    Validation,
    decode,
    encode,
};
use serde::de::{self, Visitor};

use crate::{base64, base64::BASE64_STANDARD, dev_prelude::*, public_keys::PublicKey};

pub const DEFAULT_EXP_IN_SEC: u64 = 60 * 300;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jwt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<usize>,
    pub exp: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitlements: Option<Entitlements>,
}

impl Jwt {
    pub fn builder() -> JwtBuilder {
        JwtBuilder::default()
    }

    pub fn encode(&self, key: &[u8]) -> Result<String, JwtError> {
        let key = EncodingKey::from_rsa_pem(key).map_err(JwtError::new)?;

        let result = encode(&Header::new(Algorithm::RS256), self, &key).map_err(JwtError::new)?;

        Ok(result)
    }

    pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, JwtError> {
        let key = DecodingKey::from_rsa_pem(key).map_err(JwtError::new)?;

        let result: TokenData<Jwt> =
            decode(token, &key, &Validation::new(Algorithm::RS256)).map_err(JwtError::new)?;

        Ok(result.claims)
    }

    pub fn decode_with_public_keys(token: &str, keys: &[PublicKey]) -> Result<Jwt, JwtError> {
        for key in keys {
            let res = Jwt::decode(token, key.public_key.as_ref());

            match res {
                Ok(jwt) => return Ok(jwt),
                Err(_) => continue,
            }
        }

        Err(JwtError {
            message: "no valid public key found".to_string(),
        })
    }

    /// Verify against each key trying raw PEM first (the format
    /// `GET /public_keys` serves), then base64-decoded PEM (the storage /
    /// create / find-by-id format), so the SDK works against any endpoint
    /// encoding or version skew without ever trusting an unverifiable key —
    /// a key is only ever accepted by actually validating the token.
    pub fn decode_with_flexible_public_keys(
        token: &str,
        keys: &[PublicKey],
    ) -> Result<Jwt, JwtError> {
        for key in keys {
            if let Ok(jwt) = Jwt::decode(token, key.public_key.as_ref()) {
                return Ok(jwt);
            }

            if let Ok(decoded) = BASE64_STANDARD.decode(key.public_key.as_bytes())
                && let Ok(jwt) = Jwt::decode(token, &decoded)
            {
                return Ok(jwt);
            }
        }

        Err(JwtError {
            message: "no valid public key found".to_string(),
        })
    }
}

#[derive(Debug)]
pub struct JwtError {
    message: String,
}

impl JwtError {
    pub fn new(err: impl std::error::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JwtError: {}", self.message)
    }
}

#[derive(Default)]
pub struct JwtBuilder {
    sub: Option<Uuid>,
    iss: Option<String>,
    aud: Option<String>,
    nbf: Option<Result<usize, JwtError>>,
    iat: Option<Result<usize, JwtError>>,
    ttl: Option<Duration>,
    exp: Option<usize>,
    ctx: Option<Value>,
    entitlements: Option<Result<Entitlements, JwtError>>,
}

impl JwtBuilder {
    pub fn build(self) -> Result<Jwt, JwtError> {
        let JwtBuilder {
            sub,
            iss,
            aud,
            nbf,
            iat,
            ttl,
            exp,
            ctx,
            entitlements,
        } = self;

        let now = epoch_from_time(time::SystemTime::now())?;

        let nbf = Some(nbf.unwrap_or(Ok(now - 10))?);

        let iat = Some(iat.unwrap_or(Ok(now))?);

        let exp_from_ttl = {
            let ttl = ttl.unwrap_or(Duration::from_secs(60 * 3));
            epoch_from_now(DurationDirection::Add, ttl)?
        };

        let exp = exp.unwrap_or(exp_from_ttl);

        let entitlements = entitlements.transpose()?;

        Ok(Jwt {
            sub,
            iss,
            aud,
            nbf,
            iat,
            exp,
            ctx,
            entitlements,
        })
    }

    pub fn with_subject(mut self, sub: Uuid) -> Self {
        self.sub = Some(sub);

        self
    }

    pub fn with_issuer(mut self, iss: String) -> Self {
        self.iss = Some(iss);

        self
    }

    pub fn with_audience(mut self, aud: String) -> Self {
        self.aud = Some(aud);

        self
    }

    pub fn with_not_before_from(mut self, duration: time::Duration) -> Self {
        let nbf = epoch_from_now(DurationDirection::Sub, duration);

        self.nbf = Some(nbf);

        self
    }

    pub fn with_issued_at(mut self, issued_at: time::SystemTime) -> Self {
        let iat = epoch_from_time(issued_at);

        self.iat = Some(iat);

        self
    }

    pub fn with_expires_at(mut self, exp: usize) -> Self {
        self.exp = Some(exp);

        self
    }

    pub fn with_expires_in(mut self, duration: Duration) -> Self {
        self.ttl = Some(duration);

        self
    }

    pub fn with_entitlements(
        mut self,
        encoding: EntitlementsEncoding,
        entitlements: &[String],
    ) -> Self {
        self.entitlements = Some(Entitlements::encode(encoding, entitlements));

        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementsEncoding {
    Txt,
    Gz,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Entitlements {
    /// The `txt …` claim payload: the space-joined permission list.
    Txt(String),
    /// The `gz …` claim payload: `base64(gzip(space-joined permissions))`.
    /// Both the mint side (`encode`) and the decode side (`decode` /
    /// `Deserialize` / `FromStr`) carry the wire bytes verbatim, so `Eq`,
    /// `Serialize` and JWT round-trips are symmetric (OXA-000040); read the
    /// permissions through `as_vec`, which inflates on demand.
    Gz(String),
}

pub const TXT_PREFIX: &str = "txt";
pub const GZ_PREFIX: &str = "gz";

impl Entitlements {
    pub fn encode(
        encoding: EntitlementsEncoding,
        entitlements: &[String],
    ) -> Result<Self, JwtError> {
        let entitlements = entitlements.join(" ");

        let result = match encoding {
            EntitlementsEncoding::Txt => Entitlements::Txt(entitlements),
            EntitlementsEncoding::Gz => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::best());

                encoder
                    .write_all(entitlements.as_bytes())
                    .map_err(JwtError::new)?;

                let encoded = encoder
                    .finish()
                    .map_err(JwtError::new)?;

                let based64_encoded = BASE64_STANDARD.encode(encoded);

                Entitlements::Gz(based64_encoded)
            },
        };

        Ok(result)
    }

    pub fn decode(encoded: &str) -> Result<Self, JwtError> {
        let Some((prefix, entitlemments)) = encoded.split_once(' ') else {
            return Err(JwtError {
                message: format!("malformed encoded: {}", encoded),
            });
        };

        match prefix {
            TXT_PREFIX => Ok(Self::Txt(entitlemments.to_string())),
            GZ_PREFIX => {
                // The payload is validated here — base64, gzip and UTF-8 —
                // but kept verbatim: `Gz` always holds the wire bytes, both
                // sides of the JWT boundary (OXA-000040). Malformed input
                // still fails at parse time with the same error surface;
                // `as_vec` inflates the payload on demand.
                let data = BASE64_STANDARD
                    .decode(entitlemments)
                    .map_err(JwtError::new)?;

                let mut decoder = GzDecoder::new(&*data);

                decoder
                    .read_to_string(&mut String::new())
                    .map_err(JwtError::new)?;

                Ok(Self::Gz(entitlemments.to_string()))
            },
            _ => {
                Err(JwtError {
                    message: format!("unknown entitlements prefix: {}", prefix),
                })
            },
        }
    }

    /// The permission list. Both variants hold the wire payload, so a `Gz`
    /// payload is inflated (base64 → gunzip) on demand; a payload that
    /// cannot be inflated yields `None` instead of a garbage entry.
    pub fn as_vec(&self) -> Option<Vec<String>> {
        let joined = match self {
            Entitlements::Txt(s) => Cow::Borrowed(s.as_str()),
            Entitlements::Gz(s) => {
                let data = BASE64_STANDARD
                    .decode(s)
                    .ok()?;

                let mut plain = String::new();

                GzDecoder::new(&*data)
                    .read_to_string(&mut plain)
                    .ok()?;

                Cow::Owned(plain)
            },
        };

        Some(
            joined
                .split(' ')
                .map(ToString::to_string)
                .collect(),
        )
    }
}

impl FromStr for Entitlements {
    type Err = JwtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl Serialize for Entitlements {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let text = match self {
            Entitlements::Txt(s) => format!("{} {}", TXT_PREFIX, s),
            Entitlements::Gz(s) => format!("{} {}", GZ_PREFIX, s),
        };

        serializer.serialize_str(text.as_str())
    }
}

impl<'de> Deserialize<'de> for Entitlements {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntitlementsVisitor;

        impl<'de> Visitor<'de> for EntitlementsVisitor {
            type Value = Entitlements;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a string starting with \"{}\" or \"{}\"",
                    TXT_PREFIX, GZ_PREFIX,
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Entitlements::from_str(s).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(EntitlementsVisitor)
    }
}

pub enum DurationDirection {
    Add,
    Sub,
}

pub fn epoch_from_now(direction: DurationDirection, duration: Duration) -> Result<usize, JwtError> {
    let mut now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(JwtError::new)?;

    match direction {
        DurationDirection::Add => {
            now = now.add(duration);
        },
        DurationDirection::Sub => {
            now = now.sub(duration);
        },
    }

    let expiration = now.as_secs() as usize;

    Ok(expiration)
}

pub fn epoch_from_time(t: time::SystemTime) -> Result<usize, JwtError> {
    let epoch = t
        .duration_since(UNIX_EPOCH)
        .map_err(JwtError::new)?
        .as_secs() as usize;

    Ok(epoch)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{public_keys::PublicKey, rsa::KeyPair};

    fn keypair() -> KeyPair {
        KeyPair::new().unwrap()
    }

    fn public_key_dto(pem: &[u8]) -> PublicKey {
        PublicKey {
            id: Uuid::new_v4(),
            public_key: String::from_utf8(pem.to_vec()).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn now_epoch() -> usize {
        epoch_from_now(DurationDirection::Add, Duration::ZERO).unwrap()
    }

    #[test]
    fn works_with_rsa() {
        let KeyPair { public, private } = keypair();

        let sub = Uuid::new_v4();

        let claims = Jwt::builder()
            .with_subject(sub)
            .with_issuer("oxidauth".to_string())
            .with_entitlements(
                EntitlementsEncoding::Txt,
                &[
                    "realm:resource:action".to_string(),
                    "oxidauth:**:**".to_string(),
                ],
            )
            .with_expires_in(Duration::from_secs(DEFAULT_EXP_IN_SEC))
            .build()
            .unwrap();

        let encoded = claims
            .encode(&private)
            .unwrap();

        assert_eq!(encoded.split('.').count(), 3);

        let decoded = Jwt::decode(&encoded, &public).unwrap();

        assert_eq!(decoded.sub, Some(sub));
        assert_eq!(decoded.iss.as_deref(), Some("oxidauth"));
        assert_eq!(
            decoded.entitlements,
            Some(Entitlements::Txt(
                "realm:resource:action oxidauth:**:**".to_string()
            ))
        );
        assert!(decoded.exp > decoded.iat.unwrap());
    }

    #[test]
    fn tokens_with_an_audience_claim_are_unverifiable_by_decode() {
        let KeyPair { public, private } = keypair();

        let claims = Jwt::builder()
            .with_audience("oxidauth".to_string())
            .build()
            .unwrap();

        assert_eq!(claims.aud.as_deref(), Some("oxidauth"));

        let encoded = claims
            .encode(&private)
            .unwrap();

        // BUG(pinned): `Jwt::decode` builds `Validation::new(RS256)` without an
        // expected audience, and jsonwebtoken rejects any token that *carries*
        // an `aud` when none is configured — so `with_audience` produces tokens
        // its own verifier cannot read. Production mints tokens without `aud`.
        let err = Jwt::decode(&encoded, &public).unwrap_err();

        assert_eq!(err.to_string(), "JwtError: InvalidAudience");
    }

    #[test]
    fn gz_entitlements_survive_jwt_round_trip() {
        let KeyPair { public, private } = keypair();

        let entitlements = vec![
            "oxidauth:**:**".to_string(),
            "realm:resource:action".to_string(),
        ];

        let claims = Jwt::builder()
            .with_entitlements(EntitlementsEncoding::Gz, &entitlements)
            .build()
            .unwrap();

        // before the wire trip, the Gz payload is base64-of-gzip, not plaintext
        match &claims.entitlements {
            Some(Entitlements::Gz(payload)) => assert!(payload.starts_with("H4sI")),
            other => panic!("expected gz entitlements, got {other:?}"),
        }

        let encoded = claims
            .encode(&private)
            .unwrap();

        let decoded = Jwt::decode(&encoded, &public).unwrap();

        // round-trip is identity: decoded `Gz` carries the same wire
        // payload (base64-of-gzip) as the mint side (OXA-000040)
        assert_eq!(decoded.entitlements, claims.entitlements);

        assert_eq!(
            decoded
                .entitlements
                .unwrap()
                .as_vec()
                .unwrap(),
            entitlements
        );
    }

    #[test]
    fn decode_rejects_expired_token() {
        let KeyPair { public, private } = keypair();

        // well beyond jsonwebtoken's 60s default leeway
        let past = epoch_from_now(DurationDirection::Sub, Duration::from_secs(300)).unwrap();

        let claims = Jwt::builder()
            .with_expires_at(past)
            .build()
            .unwrap();

        let encoded = claims
            .encode(&private)
            .unwrap();

        let err = Jwt::decode(&encoded, &public).unwrap_err();

        // jsonwebtoken surfaces the ErrorKind variant verbatim through the
        // JwtError envelope
        assert_eq!(err.to_string(), "JwtError: ExpiredSignature");
    }

    #[test]
    fn jsonwebtoken_default_leeway_accepts_recently_expired() {
        let KeyPair { public, private } = keypair();

        // This test exists SOLELY to depend on jsonwebtoken's default
        // `Validation::leeway` of 60s: an `exp` 30s in the past still decodes,
        // so callers must not treat `decode` as second-exact TTL enforcement.
        // If the library ever changes that default (or oxidauth starts setting
        // `leeway(0)`), this test MUST fail loudly to flag the contract change.
        let within_leeway =
            epoch_from_now(DurationDirection::Sub, Duration::from_secs(30)).unwrap();

        let claims = Jwt::builder()
            .with_expires_at(within_leeway)
            .build()
            .unwrap();

        let encoded = claims
            .encode(&private)
            .unwrap();

        assert!(
            Jwt::decode(&encoded, &public).is_ok(),
            "jsonwebtoken's default 60s leeway regressed: a token 30s past \
             `exp` no longer decodes — update TTL-sensitive call sites"
        );
    }

    #[test]
    fn encode_and_decode_reject_non_pem_keys() {
        let claims = Jwt::builder()
            .build()
            .unwrap();

        assert!(
            claims
                .encode(b"not-a-pem-key")
                .is_err()
        );
        assert!(Jwt::decode("a.b.c", b"not-a-pem-key").is_err());
    }

    #[test]
    fn decode_with_public_keys_tries_each_key() {
        let signing = keypair();
        let foreign = keypair();

        let claims = Jwt::builder()
            .build()
            .unwrap();

        let encoded = claims
            .encode(&signing.private)
            .unwrap();

        // a foreign key alone cannot verify the token
        assert!(Jwt::decode(&encoded, &foreign.public).is_err());

        // the matching key is found even when it trails a foreign one
        let decoded = Jwt::decode_with_public_keys(
            &encoded,
            &[
                public_key_dto(&foreign.public),
                public_key_dto(&signing.public),
            ],
        )
        .unwrap();

        assert_eq!(decoded.exp, claims.exp);

        // only foreign keys / no keys at all -> the sentinel error
        let err =
            Jwt::decode_with_public_keys(&encoded, &[public_key_dto(&foreign.public)]).unwrap_err();
        assert_eq!(err.to_string(), "JwtError: no valid public key found");

        let err = Jwt::decode_with_public_keys(&encoded, &[]).unwrap_err();
        assert_eq!(err.to_string(), "JwtError: no valid public key found");
    }

    #[test]
    fn decode_with_flexible_public_keys_tries_raw_pem_then_base64() {
        let signing = keypair();
        let foreign = keypair();

        let claims = Jwt::builder()
            .build()
            .unwrap();

        let encoded = claims
            .encode(&signing.private)
            .unwrap();

        // raw PEM — the format `GET /public_keys` serves — verifies on the
        // primary leg, even trailing a foreign key
        let decoded = Jwt::decode_with_flexible_public_keys(
            &encoded,
            &[
                public_key_dto(&foreign.public),
                public_key_dto(&signing.public),
            ],
        )
        .unwrap();
        assert_eq!(decoded.exp, claims.exp);

        // base64(PEM) — the storage / create / find-by-id encoding — verifies
        // on the fallback leg
        let signing_b64 = BASE64_STANDARD.encode(&signing.public);
        let decoded = Jwt::decode_with_flexible_public_keys(
            &encoded,
            &[public_key_dto(signing_b64.as_bytes())],
        )
        .unwrap();
        assert_eq!(decoded.exp, claims.exp);

        // garbage neither decodable as PEM nor as base64, plus base64 of
        // non-PEM bytes: both legs pass the failure through and the walk
        // continues (no panic), the matching key still wins
        let not_pem_b64 = BASE64_STANDARD.encode(b"definitely not a pem key");
        let decoded = Jwt::decode_with_flexible_public_keys(
            &encoded,
            &[
                public_key_dto(b"-----BEGIN PUBLIC KEY-----\n!!! not base64 !!!\n-----END"),
                public_key_dto(not_pem_b64.as_bytes()),
                public_key_dto(&signing.public),
            ],
        )
        .unwrap();
        assert_eq!(decoded.exp, claims.exp);

        // foreign keys in BOTH encodings cannot verify -> the sentinel error;
        // the fallback must not rubber-stamp decodable-but-wrong keys
        let foreign_b64 = BASE64_STANDARD.encode(&foreign.public);
        let err = Jwt::decode_with_flexible_public_keys(
            &encoded,
            &[
                public_key_dto(&foreign.public),
                public_key_dto(foreign_b64.as_bytes()),
            ],
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "JwtError: no valid public key found");

        // a base64 key that decodes to non-PEM: pass-through error, no panic
        let err = Jwt::decode_with_flexible_public_keys(
            &encoded,
            &[public_key_dto(not_pem_b64.as_bytes())],
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "JwtError: no valid public key found");

        // empty key set -> the sentinel error
        let err = Jwt::decode_with_flexible_public_keys(&encoded, &[]).unwrap_err();
        assert_eq!(err.to_string(), "JwtError: no valid public key found");
    }

    #[test]
    fn builder_applies_nbf_iat_and_ttl_defaults() {
        let before = now_epoch();

        let jwt = Jwt::builder()
            .build()
            .unwrap();

        let after = now_epoch();

        let iat = jwt.iat.unwrap();

        assert!((before..=after).contains(&iat));

        // default nbf is `now - 10s`, derived from the same clock reading as iat
        assert_eq!(jwt.nbf, Some(iat - 10));

        // default ttl is 3 minutes, read from a second clock sample
        let delta = jwt.exp as i64 - iat as i64;
        assert!((178..=182).contains(&delta), "exp - iat was {delta}s");
    }

    #[test]
    fn builder_honors_explicit_claims() {
        let sub = Uuid::new_v4();

        let issued_at = UNIX_EPOCH + Duration::from_secs(1_000_000_000);

        let before = now_epoch();

        let jwt = Jwt::builder()
            .with_subject(sub)
            .with_issued_at(issued_at)
            .with_not_before_from(Duration::from_secs(60))
            .with_expires_at(2_000_000_000)
            .build()
            .unwrap();

        let after = now_epoch();

        assert_eq!(jwt.sub, Some(sub));
        assert_eq!(jwt.iat, Some(1_000_000_000));
        assert_eq!(jwt.exp, 2_000_000_000);

        // `with_not_before_from` subtracts from now, so nbf lands ~60s in the past
        assert!((before - 61..=after - 59).contains(&jwt.nbf.unwrap()));
    }

    #[test]
    fn epoch_from_now_orders_sub_before_add() {
        let now = now_epoch();

        let sub = epoch_from_now(DurationDirection::Sub, Duration::from_secs(120)).unwrap();

        let add = epoch_from_now(DurationDirection::Add, Duration::from_secs(60)).unwrap();

        assert!(sub <= now, "Sub must land at or before now");
        assert!(add >= now, "Add must land at or after now");

        let delta = add as i64 - sub as i64;
        assert!((178..=182).contains(&delta), "add - sub was {delta}s");
    }

    #[test]
    fn epoch_from_time_converts_system_time() {
        assert_eq!(epoch_from_time(UNIX_EPOCH).unwrap(), 0);
        assert_eq!(
            epoch_from_time(UNIX_EPOCH + Duration::from_secs(10)).unwrap(),
            10
        );
    }

    #[test]
    fn txt_entitlements_wire_format_is_prefix_plus_joined_list() {
        let entitlements = vec!["a:b:c".to_string(), "d:e:f".to_string()];

        let txt = Entitlements::encode(EntitlementsEncoding::Txt, &entitlements).unwrap();

        assert_eq!(txt, Entitlements::Txt("a:b:c d:e:f".to_string()));

        assert_eq!(
            serde_json::to_value(&txt).unwrap(),
            json!("txt a:b:c d:e:f")
        );

        assert_eq!(
            serde_json::from_value::<Entitlements>(json!("txt a:b:c d:e:f")).unwrap(),
            txt
        );

        // encoding tags themselves ride the wire as snake_case tokens
        assert_eq!(
            serde_json::to_value(EntitlementsEncoding::Txt).unwrap(),
            json!("txt")
        );
        assert_eq!(
            serde_json::to_value(EntitlementsEncoding::Gz).unwrap(),
            json!("gz")
        );
    }

    #[test]
    fn gz_entitlements_wire_format_round_trips_via_from_str() {
        let entitlements = vec!["a:b:c".to_string(), "d:e:f".to_string()];

        let gz = Entitlements::encode(EntitlementsEncoding::Gz, &entitlements).unwrap();

        let wire = serde_json::to_value(&gz).unwrap();

        let wire = wire.as_str().unwrap();

        assert!(
            wire.starts_with("gz H4sI"),
            "unexpected gz wire form: {wire}"
        );

        // FromStr delegates to `decode`: the wire payload comes back
        // verbatim, equal to the encode-side value (OXA-000040)
        let parsed: Entitlements = wire.parse().unwrap();

        assert_eq!(parsed, gz);
        assert_eq!(parsed.as_vec().unwrap(), entitlements);
    }

    #[test]
    fn decoded_gz_claims_re_sign_to_redecodable_tokens() {
        let KeyPair { public, private } = keypair();

        let entitlements = vec![
            "oxidauth:**:**".to_string(),
            "realm:resource:action".to_string(),
        ];

        let claims = Jwt::builder()
            .with_entitlements(EntitlementsEncoding::Gz, &entitlements)
            .build()
            .unwrap();

        let original_claim = serde_json::to_value(&claims.entitlements).unwrap();

        let first = Jwt::decode(
            &claims
                .encode(&private)
                .unwrap(),
            &public,
        )
        .unwrap();

        // re-issue from the decoded claim WITHOUT rebuilding the entitlements:
        // the decoded `Gz` must serialize back to the wire form, or the
        // re-signed token carries a valid RS256 signature whose own claims
        // fail base64/gzip decoding at the next `Jwt::decode`
        let redecoded = Jwt::decode(
            &first
                .encode(&private)
                .unwrap(),
            &public,
        )
        .unwrap();

        assert_eq!(
            redecoded
                .entitlements
                .as_ref()
                .unwrap()
                .as_vec()
                .unwrap(),
            entitlements
        );

        // the decoded claim serializes back to the byte-identical wire claim
        assert_eq!(
            serde_json::to_value(&first.entitlements).unwrap(),
            original_claim
        );
    }

    #[test]
    fn entitlements_decode_rejects_malformed_input() {
        // no prefix separator
        let err = Entitlements::decode("just-one-token").unwrap_err();
        assert_eq!(
            err.to_string(),
            "JwtError: malformed encoded: just-one-token"
        );

        // unknown prefix
        let err = Entitlements::decode("unknown payload").unwrap_err();
        assert_eq!(
            err.to_string(),
            "JwtError: unknown entitlements prefix: unknown"
        );

        // gz prefix with non-base64 payload
        assert!(Entitlements::decode("gz !!!not-base64!!!").is_err());
    }
}
