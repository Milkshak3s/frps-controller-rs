use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),

    #[error("Kube Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("Finalizer Error: {0}")]
    FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),

    #[error("IllegalDocument")]
    IllegalDocument,
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod endpoints;
pub use crate::endpoints::*;

pub mod frpclients;
pub use crate::frpclients::*;
