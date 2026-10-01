use super::{Config, OxidauthClient};

pub struct OxidauthClientBuilder {
    host: Option<String>,
    config: Option<Config>,
}

impl Default for OxidauthClientBuilder {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    // E6: the validation logic of `build()` is natively testable because both
    // error arms return *before* `OxidauthClient::new` touches LocalStorage
    // (gloo-storage panics via js-sys on non-wasm targets). The success arm
    // constructs `OxidauthClient::new`, i.e. hits `State::load` ->
    // LocalStorage::get, so it is NOT natively testable — pinned instead by
    // `client_construction_panics_on_native` in `wasm/mod.rs`.

    #[test]
    fn build_without_host_reports_the_host_error() {
        let err = OxidauthClientBuilder::new()
            .build()
            .err()
            .unwrap();
        assert_eq!(err, "OxidauthClient requires a host");
    }

    #[test]
    fn build_without_config_reports_the_config_error() {
        let err = OxidauthClientBuilder::new()
            .host("https://oxidauth.test".to_string())
            .build()
            .err()
            .unwrap();
        assert_eq!(err, "OxidauthClient requires a config");
    }

    #[test]
    fn host_is_validated_before_config() {
        let err = OxidauthClientBuilder::new()
            .config(Config::default())
            .build()
            .err()
            .unwrap();
        assert_eq!(
            err, "OxidauthClient requires a host",
            "a config-set-but-hostless builder must still report the host gap"
        );
    }
}
