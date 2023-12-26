use std::error::Error;

use rsa::{
    pkcs1v15::Pkcs1v15Encrypt,
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    traits::PaddingScheme,
    RsaPrivateKey, RsaPublicKey,
};

const DEFAULT_BIT_SIZE: usize = 4096;

pub struct KeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

#[derive(Debug)]
pub struct RsaError {}

impl KeyPair {
    pub fn new() -> Result<Self, RsaError> {
        let mut rng = rand::thread_rng();

        let private_key =
            RsaPrivateKey::new(&mut rng, DEFAULT_BIT_SIZE).map_err(|_| RsaError {})?;

        let public_key = RsaPublicKey::from(&private_key);

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|_| RsaError {})?;

        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|_| RsaError {})?;

        let private: Vec<u8> = private_pem.to_string().into();

        let public: Vec<u8> = public_pem.into();

        Ok(KeyPair { public, private })
    }

    pub fn base64_encode(&self) -> Base64KeyPair {
        let public: Vec<u8> = base64::encode(&self.public).into();
        let private: Vec<u8> = base64::encode(&self.private).into();

        Base64KeyPair { public, private }
    }
}

pub struct Base64KeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

impl From<(&[u8], &[u8])> for Base64KeyPair {
    fn from((public, private): (&[u8], &[u8])) -> Self {
        let public = public.to_vec();
        let private = private.to_vec();

        Base64KeyPair { public, private }
    }
}

pub struct PublicKey(RsaPublicKey);

impl PublicKey {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut rng = rand::thread_rng();

        let data = self.0.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;

        Ok(data)
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = Box<dyn Error>;

    fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
        let raw: &[u8] = raw.as_ref();
        raw.try_into()
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = Box<dyn Error>;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        let pem = String::from_utf8(base64::decode(raw)?)?;
        let key = RsaPublicKey::from_public_key_pem(&pem)?;

        Ok(PublicKey(key))
    }
}

pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let data = self.0.decrypt(Pkcs1v15Encrypt, &data)?;

        Ok(data)
    }
}

impl TryFrom<Vec<u8>> for PrivateKey {
    type Error = Box<dyn Error>;

    fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
        let raw: &[u8] = raw.as_ref();
        raw.try_into()
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = Box<dyn Error>;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        let pem = String::from_utf8(base64::decode(raw)?)?;
        let key = RsaPrivateKey::from_pkcs8_pem(&pem)?;

        Ok(PrivateKey(key))
    }
}

// #[cfg(test)]
// #[cfg(feature = "rsa-test")]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_encrypt_and_decrypt() {
//         let Base64KeyPair { public, private } = generate().unwrap().base64_encode();
//
//         let data = b"hello world!";
//
//         let private: PrivateKey = private.try_into().unwrap();
//         let public: PublicKey = public.try_into().unwrap();
//
//         let encrypted = public.encrypt(data).unwrap();
//
//         let decrypted = private.decrypt(&encrypted).unwrap();
//
//         assert_eq!(decrypted, data);
//     }
// }
