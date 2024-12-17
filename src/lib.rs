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

pub mod client;
pub mod config;
pub mod err;

use client::MLFlowClient;
use err::MLFlowError::*;

use crate::client::MLFLowRestClient;
use serde::{Deserialize, Serialize};

pub type MLFlowResult<T> = Result<T, err::MLFlowError>;

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

pub enum ExperimentIdentifierType {
    ById(String),
    ByName(String),
}

#[derive(Default)]
pub struct ExperimentLoader {
    client: Option<MLFLowRestClient>,
}

impl ExperimentLoader {
    pub fn with_client(mut self, client: MLFLowRestClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn load(self, experiment_identifier: ExperimentIdentifierType) -> MLFlowResult<Experiment> {
        let client: MLFLowRestClient = self
            .client
            .unwrap_or_else(|| MLFLowRestClient::new("http://localhost:5000"));

        match experiment_identifier {
            ExperimentIdentifierType::ById(id) => match client.get_experiment_by_id(id) {
                Ok(resp) => Ok(resp.experiment),
                Err(e) => Err(UnknownError(e.to_string())),
            },
            ExperimentIdentifierType::ByName(name) => match client.get_experiment_by_name(name) {
                Ok(resp) => Ok(resp.experiment),
                Err(e) => Err(e),
            },
        }
    }
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
