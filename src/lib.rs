//! Rust implementation of a MLFlow Tracking Server Client
//!
//! # Usage
//!
//! ```rust
//!     use mlflow_rs::{Experiment, ExperimentBuilder};
//!
//!     let experiment = ExperimentBuilder::new("my-ml-experiment".to_string());
//!
//!
//! ```
//!

use crate::MLFlowError::{ExperimentBuilderError, UnknownError};
use log::debug;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub type MLFlowResult<T> = Result<T, MLFlowError>;
#[derive(thiserror::Error, Debug, Clone)]
pub enum MLFlowError {
    #[error("ExperimentBuilderError: {0}")]
    ExperimentBuilderError(String),

    #[error("ClientError: {0}")]
    ClientError(String),

    #[error("ResourceAlreadyExists: {0}")]
    ResourceAlreadyExists(String),

    #[error("UnknownError: {0}")]
    UnknownError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentTag {
    key: String,
    value: String,
}

impl From<(&str, &str)> for ExperimentTag {
    fn from((k, v): (&str, &str)) -> Self {
        ExperimentTag {
            key: k.to_string(),
            value: v.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateExperimentResponse {
    experiment_id: String,
}

trait MLFlowClient {
    fn create_experiment(&self, experiment: Experiment) -> MLFlowResult<CreateExperimentResponse>;
}

#[derive(Clone, Debug, Default)]
pub struct MLFLowRestClient {
    client: Client,
    host: String,
}

impl MLFLowRestClient {
    pub fn new(host: impl AsRef<str>) -> Self {
        //TODO support resolvers for host

        MLFLowRestClient {
            client: Client::new(),
            host: host.as_ref().to_string(),
        }
    }
}

impl MLFlowClient for MLFLowRestClient {
    fn create_experiment(&self, experiment: Experiment) -> MLFlowResult<CreateExperimentResponse> {
        let url = format!("{}{}", &self.host, "/api/2.0/mlflow/experiments/create");
        let result = self.client.post(url).json(&experiment).send();

        match result {
            Ok(result) => {
                if result.status().is_success() {
                    let e = result.json::<CreateExperimentResponse>();
                    match e {
                        Ok(result) => Ok(result),
                        Err(e) => Err(UnknownError(e.to_string())),
                    }
                } else {
                    println!("{:?}", result.error_for_status());
                    Err(UnknownError("Could not create experiment".to_string()))
                }
            }
            Err(result) => {
                debug!("{}", result.to_string());
                Err(UnknownError(result.to_string()))
            }
        }
    }
}

pub trait ExperimentIdentifier {
    fn experiment_id(&self) -> Option<String>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Experiment {
    experiment_id: Option<String>,
    pub name: String,
    pub artifact_location: Option<String>,
    pub tags: Vec<ExperimentTag>,

    #[serde(skip_serializing, skip_deserializing)]
    #[allow(dead_code)]
    client: MLFLowRestClient,
}

impl ExperimentIdentifier for Experiment {
    fn experiment_id(&self) -> Option<String> {
        self.experiment_id.clone()
    }
}

#[derive(Clone, Debug)]

pub struct ExperimentBuilder {
    name: String,
    artifact_location: Option<String>,
    tags: Vec<ExperimentTag>,
    client: MLFLowRestClient,
}

impl ExperimentBuilder {
    pub fn new(name: impl AsRef<str>) -> MLFlowResult<ExperimentBuilder> {
        if name.as_ref().is_empty() {
            return Err(ExperimentBuilderError("name cannot be empty".to_string()));
        }

        Ok(ExperimentBuilder {
            name: name.as_ref().to_string(),
            artifact_location: None,
            tags: vec![],
            client: MLFLowRestClient::new("http://localhost:5000"),
        })
    }

    pub fn with_tag(mut self, tag: impl Into<ExperimentTag>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<impl Into<ExperimentTag>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn with_rest_client(mut self, client: MLFLowRestClient) -> Self {
        self.client = client;
        self
    }

    pub fn build(self) -> MLFlowResult<Experiment> {
        //TODO Get a count of tags that have either an empty key or empty value
        let client = self.client.clone();
        let mut e = Experiment {
            experiment_id: None,
            name: self.name,
            artifact_location: self.artifact_location.clone(),
            tags: self.tags.clone(),
            client: self.client.clone(),
        };

        let result = client.create_experiment(e.clone());

        match result {
            Ok(resp) => {
                e.experiment_id = Some(resp.experiment_id);
                Ok(e)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Version {
    version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experiment_builder_new_empty_name() {
        ExperimentBuilder::new("").expect_err("ExperimentBuilderError: name cannot be empty");
    }

    #[test]
    fn builder_with_tag() {
        let builder = ExperimentBuilder::new("test_experiment")
            .unwrap()
            .with_tag(("key", "value"))
            .with_tag(("key2", "value2"));

        assert_eq!(builder.name, "test_experiment");
        assert_eq!(builder.tags.len(), 2);
    }

    #[test]
    fn tuple_to_experiment_tag() {
        let tag = ExperimentTag::from(("name", "value"));
        assert_eq!(tag.key, "name");
        assert_eq!(tag.value, "value");
    }

    #[test]
    fn build_with_tags() {
        let builder = ExperimentBuilder::new("test_experiment")
            .unwrap()
            .with_tags(vec![("key", "value"), ("key2", "value2")]);
        assert_eq!(builder.name, "test_experiment");
        assert_eq!(builder.tags.len(), 2);
    }
}
