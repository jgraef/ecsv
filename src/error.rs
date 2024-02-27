#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("yaml error")]
    Yaml(#[from] serde_yaml::Error),

    #[error("invalid signature")]
    InvalidSignature,
}
