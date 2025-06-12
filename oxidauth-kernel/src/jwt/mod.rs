use std::fmt;
use std::io::prelude::*;
use std::ops::{Add as _, Sub as _};
use std::str::FromStr;
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};

use crate::base64::BASE64_STANDARD;
use base64::Engine;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::de::{self, Visitor};

use crate::public_keys::PublicKey;
use crate::{base64, dev_prelude::*};

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

        let result = encode(
            &Header::new(Algorithm::RS256),
            self,
            &key,
        )
        .map_err(JwtError::new)?;

        Ok(result)
    }

    pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, JwtError> {
        let key = DecodingKey::from_rsa_pem(key).map_err(JwtError::new)?;

        let result: TokenData<Jwt> = decode(
            token,
            &key,
            &Validation::new(Algorithm::RS256),
        )
        .map_err(JwtError::new)?;

        Ok(result.claims)
    }

    pub fn decode_with_public_keys(
        token: &str,
        keys: &[PublicKey],
    ) -> Result<Jwt, JwtError> {
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
        write!(
            f,
            "JwtError: {}",
            self.message
        )
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
        self.entitlements = Some(Entitlements::encode(
            encoding,
            entitlements,
        ));

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
    Txt(String),
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
                let mut encoder = GzEncoder::new(
                    Vec::new(),
                    Compression::best(),
                );

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
                message: format!(
                    "malformed encoded: {}",
                    encoded
                ),
            });
        };

        match prefix {
            TXT_PREFIX => Ok(Self::Txt(
                entitlemments.to_string(),
            )),
            GZ_PREFIX => {
                let data = BASE64_STANDARD
                    .decode(entitlemments)
                    .map_err(JwtError::new)?;

                let mut decoder = GzDecoder::new(&*data);
                let mut decoded = String::new();

                decoder
                    .read_to_string(&mut decoded)
                    .map_err(JwtError::new)?;

                Ok(Self::Gz(decoded))
            },
            _ => Err(JwtError {
                message: format!(
                    "unknown entitlements prefix: {}",
                    prefix
                ),
            }),
        }
    }

    pub fn as_vec(&self) -> Option<Vec<String>> {
        let joined = match self {
            Entitlements::Txt(ref s) => s,
            Entitlements::Gz(ref s) => s,
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

// #[cfg(test)]
// mod tests {
//     use crate::rsa::{generate, KeyPair};
//
//     use super::*;
//
//     #[test]
//     fn works_with_rsa() {
//         let KeyPair { public, private } = generate().unwrap();
//
//         let claims = Jwt::builder()
//             .with_entitlements(vec!["realm:resource:action".to_string()])
//             .with_expires_in(Duration::from_secs(60 * 300))
//             .build()
//             .unwrap();
//
//         let encoded = claims.encode(&private).unwrap();
//
//         let decoded = Jwt::decode(&encoded, &public).unwrap();
//
//         assert_eq!(
//             decoded.entitlements,
//             Some("realm:resource:action".to_string())
//         )
//     }
// }
