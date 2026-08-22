use super::{OxidauthClient, OxidauthClientConfig};

pub struct OxidauthClientBuilder {
    host: Option<String>,
    config: Option<OxidauthClientConfig>,
}

impl OxidauthClientBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            config: None,
        }
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn config(mut self, config: OxidauthClientConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<OxidauthClient, String> {
        Ok(OxidauthClient::new(self.host, self.config))
    }
}
