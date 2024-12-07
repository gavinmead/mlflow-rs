use mlflow_rs::add;
use rstest::*;
use std::time::Duration;
use testcontainers::core::ContainerPort::Tcp;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::SyncRunner,
    ContainerRequest, GenericImage, ImageExt,
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

#[test]
fn test_add() {
    assert_eq!(add(3, 2), 5);
}

#[rstest]
fn test_fixture_ok(mlflow_server_container: ContainerRequest<GenericImage>) {
    let container = mlflow_server_container.start().unwrap();
    let host_port = container.get_host_port_ipv4(Tcp(5000)).unwrap();
    let url = format!("http://localhost:{}/{}", host_port, "version");
    let mut resResult = reqwest::blocking::get(url);
    assert_eq!(true, resResult.is_ok());
    let res = resResult.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let version = res.text().unwrap();
    assert_eq!(version, MLFLOW_VERSION);
}
