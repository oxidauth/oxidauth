use std::fmt;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Client {
    inner: Arc<RwLock<Inner>>,
}

impl Client {
    pub fn new() -> Result<Self, ClientError> {
        todo!()
    }

    pub async fn authenticate(&self) -> Result<bool, ClientError> {
        Ok(true)
    }

    pub async fn refresh(&self) -> Result<bool, ClientError> {
        Ok(true)
    }

    async fn handle_auth_response(&self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn read_check(&self) -> ReadCheck {
        ReadCheck::Valid
    }

    pub async fn request<Req, Res>(
        &self,
        method: Method,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        todo!()
    }

    pub async fn get<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::GET, url, payload)
            .await
    }

    pub async fn post<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::POST, url, payload)
            .await
    }

    pub async fn delete<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::DELETE, url, payload)
            .await
    }
}

#[derive(Debug)]
pub struct ClientError {
    pub kind: ClientErrorKind,
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

#[derive(Debug)]
pub enum ClientErrorKind {
    AuthError,
    RefreshError,
    Other(&'static str),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClientErrorKind::*;

        match self.kind {
            AuthError => write!(
                f,
                "encountered an error authenticating"
            ),
            RefreshError => write!(
                f,
                "encountered an error while refreshing token"
            ),
            Other(reason) => write!(f, "error: {}", reason),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(err) => Some(err.as_ref()),
            None => None,
        }
    }
}

#[derive(Debug)]
enum ReadCheck {
    Auth,
    Valid,
}

#[derive(Debug)]
struct Inner {
    base_url: String,
    username: String,
    password: String,
    client: reqwest::Client,
    jwt: String,
    expires_at: Option<DateTime<Utc>>,
}
