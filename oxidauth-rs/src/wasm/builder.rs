use super::{Config, OxidauthClient};

pub struct OxidauthClientBuilder {
    host: Option<String>,
    config: Option<Config>,
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

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<OxidauthClient, String> {
        let Some(host) = self.host else {
            return Err("OxidauthClient requires a host".into());
        };

        let Some(config) = self.config else {
            return Err("OxidauthClient requires a config".into());
        };

        Ok(OxidauthClient::new(host, config))
    }
}
