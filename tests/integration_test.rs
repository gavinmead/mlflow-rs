use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rstest::*;
use testcontainers::core::ContainerPort::Tcp;
use testcontainers::{
    core::WaitFor, runners::SyncRunner, ContainerRequest, GenericImage, ImageExt,
};

use mlflow_rs::ExperimentIdentifierType::{ById, ByName};
use mlflow_rs::{
    ExperimentBuilder, ExperimentIdentifier, ExperimentLoader, MLFLowRestClient, MLFlowError,
};

const MLFLOW_VERSION: &str = "2.18.0";
const MLFLOW_DOCKER_IMAGE: &str = "ghcr.io/mlflow/mlflow";
const MLFLOW_ENTRY_POINT: &str = "mlflow";

/*
Using a provided docker image, constructs the url needed to connect to an MLFlow Server for testing
 */
#[fixture]
fn mlflow_server_container() -> ContainerRequest<GenericImage> {
    let docker_version = format!("v{}", MLFLOW_VERSION);
    let container = GenericImage::new(MLFLOW_DOCKER_IMAGE, docker_version.as_str())
        .with_wait_for(WaitFor::seconds(5))
        .with_entrypoint(MLFLOW_ENTRY_POINT)
        .with_exposed_port(Tcp(5000))
        .with_cmd(["server", "--host", "0.0.0.0", "--port", "5000"]);

    container
}

#[fixture]
fn experiment_name() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();

    format!("experiment-{}", rand_string)
}

#[rstest]
fn test_fixture_ok(mlflow_server_container: ContainerRequest<GenericImage>) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}/{}", host_port, "version");
    let res_result = reqwest::blocking::get(url);
    assert!(res_result.is_ok());
    let res = res_result.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let version = res.text().unwrap();
    assert_eq!(version, MLFLOW_VERSION);
}

#[rstest]
fn test_create_experiment(mlflow_server_container: ContainerRequest<GenericImage>) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}", host_port);

    let client = MLFLowRestClient::new(url);

    let experiment = ExperimentBuilder::new("test-experiment")
        .unwrap()
        .with_tag(("tag1", "value1"))
        .with_rest_client(client)
        .build();
    assert!(experiment.is_ok());
    assert!(experiment.clone().unwrap().experiment_id().is_some());
    let id = experiment.clone().unwrap().experiment_id().unwrap();
    println!("{}", id);
}

#[rstest]
fn test_create_and_get_by_id_ok(
    mlflow_server_container: ContainerRequest<GenericImage>,
    experiment_name: String,
) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}", host_port);
    println!("{}", experiment_name.clone());

    let client = MLFLowRestClient::new(url);
    let experiment = ExperimentBuilder::new(experiment_name)
        .unwrap()
        .with_tag(("tag1", "value1"))
        .with_rest_client(client.clone())
        .build();
    assert!(experiment.clone().is_ok());

    let id = experiment.clone().unwrap().experiment_id().unwrap();
    println!("{}", id.clone());

    let found_experiment = ExperimentLoader::default()
        .with_client(client.clone())
        .load(ById(id.clone()));
    assert!(found_experiment.clone().is_ok());

    let found_experiment_id = found_experiment.unwrap().experiment_id().unwrap();

    assert_eq!(found_experiment_id, id);
}

#[rstest]
fn test_create_and_get_by_name_ok(
    mlflow_server_container: ContainerRequest<GenericImage>,
    experiment_name: String,
) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}", host_port);
    println!("{}", experiment_name.clone());

    let client = MLFLowRestClient::new(url);
    let experiment = ExperimentBuilder::new(experiment_name.clone())
        .unwrap()
        .with_tag(("tag1", "value1"))
        .with_rest_client(client.clone())
        .build();
    assert!(experiment.clone().is_ok());

    let id = experiment.clone().unwrap().experiment_id().unwrap();
    println!("{}", id.clone());

    let found_experiment = ExperimentLoader::default()
        .with_client(client.clone())
        .load(ByName(experiment_name.clone()));
    assert!(found_experiment.clone().is_ok());

    let found_experiment_id = found_experiment.unwrap().experiment_id().unwrap();
    assert_eq!(found_experiment_id, id);
}

#[rstest]
fn test_get_by_name_not_found(
    mlflow_server_container: ContainerRequest<GenericImage>,
    experiment_name: String,
) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}", host_port);
    println!("{}", experiment_name.clone());

    let client = MLFLowRestClient::new(url);
    let found_experiment = ExperimentLoader::default()
        .with_client(client.clone())
        .load(ByName(experiment_name.clone()));
    assert!(matches!(
        found_experiment.err().unwrap(),
        MLFlowError::ExperimentNotFound(s) if s == "experiment was not found"
    ));
}
