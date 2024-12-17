#[derive(thiserror::Error, Debug, Clone)]
pub enum MLFlowError {
    #[error("ExperimentBuilderError: {0}")]
    ExperimentBuilderError(String),

    #[error("{0}")]
    ExperimentNotFound(String),

    #[error("ClientError: {0}")]
    ClientError(String),

    #[error("ResourceAlreadyExists: {0}")]
    ResourceAlreadyExists(String),

    #[error("UnknownError: {0}")]
    UnknownError(String),

    #[error("{0}")]
    ConfigError(String),
}
