use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("{0}")]
    Msg(String),
}
