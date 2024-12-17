use crate::client::MLFLowRestClient;
use crate::err::MLFlowError::ConfigError;
use crate::MLFlowResult;

pub struct Config {
    tracking_server_uri: String,
    client: MLFLowRestClient,
}

impl Config {
    pub fn default() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn get_client(&self) -> &MLFLowRestClient {
        &self.client
    }

    pub fn get_tracking_server_uri(&self) -> &str {
        &self.tracking_server_uri
    }
}

struct ConfigBuilder {
    tracking_server_uri: Option<String>,
}

impl ConfigBuilder {
    fn default() -> ConfigBuilder {
        Self {
            tracking_server_uri: Some(String::from("http://localhost:5000")),
        }
    }

    fn with_tracking_server_uri(mut self, tracking_server_uri: impl AsRef<str>) -> ConfigBuilder {
        self.tracking_server_uri = Some(tracking_server_uri.as_ref().to_owned());
        self
    }

    fn build(self) -> Config {
        self.try_build().unwrap()
    }

    fn try_build(self) -> MLFlowResult<Config> {
        //check the tracking_server_uri is not empty
        match self.tracking_server_uri.clone() {
            Some(uri) => {
                if uri.is_empty() {
                    return Err(ConfigError(String::from("empty tracking server uri")));
                }
            }
            None => {
                return Err(ConfigError("tracking server uri was not set".to_string()));
            }
        }

        //build the client
        let client = MLFLowRestClient::new(self.tracking_server_uri.clone().unwrap());
        Ok(Config {
            tracking_server_uri: self.tracking_server_uri.clone().unwrap(),
            client,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::MLFlowError;

    #[test]
    fn test_tracking_server_uri() {
        let cb = ConfigBuilder::default().with_tracking_server_uri("http://localhost:5001");
        assert!(cb.tracking_server_uri.is_some());
        assert_eq!(cb.tracking_server_uri.unwrap(), "http://localhost:5001");
    }

    #[test]
    fn test_default_config() {
        let cfg = ConfigBuilder::default().build();
        assert_eq!(cfg.tracking_server_uri, "http://localhost:5000");
    }

    #[test]
    fn test_custom_config() {
        let cfg = ConfigBuilder::default()
            .with_tracking_server_uri("http://localhost:5001")
            .build();
        assert_eq!(cfg.tracking_server_uri, "http://localhost:5001");
        assert_eq!(cfg.get_tracking_server_uri(), "http://localhost:5001");
    }

    #[test]
    fn test_bad_tracking_server_uri() {
        let cb = ConfigBuilder::default()
            .with_tracking_server_uri("")
            .try_build();
        assert!(cb.is_err());

        assert!(matches!(
                cb.err().unwrap(),
                MLFlowError::ConfigError(s) if s == "empty tracking server uri"
        ));
    }
}
