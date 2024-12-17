use crate::err::MLFlowError::{ExperimentNotFound, UnknownError};
use crate::{Experiment, MLFlowResult};
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateExperimentResponse {
    pub experiment_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetExperimentResponse {
    pub experiment: Experiment,
}

pub trait MLFlowClient {
    fn create_experiment(&self, experiment: Experiment) -> MLFlowResult<CreateExperimentResponse>;

    fn get_experiment_by_id(&self, id: impl AsRef<str>) -> MLFlowResult<GetExperimentResponse>;

    fn get_experiment_by_name(&self, name: impl AsRef<str>) -> MLFlowResult<GetExperimentResponse>;
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

    fn _process_get(
        &self,
        result: Result<Response, reqwest::Error>,
    ) -> MLFlowResult<GetExperimentResponse> {
        match result {
            Ok(r) => {
                if r.status().is_success() {
                    let e = r.json::<GetExperimentResponse>();
                    match e {
                        Ok(result) => Ok(result),
                        Err(e) => {
                            println!("{}", e);
                            Err(UnknownError(e.to_string()))
                        }
                    }
                } else if r.status() == StatusCode::NOT_FOUND {
                    Err(ExperimentNotFound("experiment was not found".to_string()))
                } else {
                    println!("experiment not found server message: {}", r.status());
                    Err(UnknownError("error finding experiment".to_string()))
                }
            }
            Err(e) => {
                println!("{}", e);
                Err(UnknownError(e.to_string()))
            }
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
                println!("{}", result);
                Err(UnknownError(result.to_string()))
            }
        }
    }

    fn get_experiment_by_id(&self, id: impl AsRef<str>) -> MLFlowResult<GetExperimentResponse> {
        let url = format!("{}{}", &self.host, "/api/2.0/mlflow/experiments/get");
        let result = self
            .client
            .get(url)
            .query(&[("experiment_id", id.as_ref())])
            .send();
        self._process_get(result)
    }

    fn get_experiment_by_name(&self, name: impl AsRef<str>) -> MLFlowResult<GetExperimentResponse> {
        let url = format!(
            "{}{}",
            &self.host, "/api/2.0/mlflow/experiments/get-by-name"
        );
        let result = self
            .client
            .get(url)
            .query(&[("experiment_name", name.as_ref())])
            .send();
        self._process_get(result)
    }
}
