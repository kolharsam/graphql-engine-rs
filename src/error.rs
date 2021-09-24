use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Clone)]
pub enum GQLRSErrorType {
    #[error("ERROR: `{0}`")]
    GenericError(String),
    #[error("ERROR: Table `{0}` is already tracked")]
    TableAlreadyTracked(String),
    #[error("ERROR: Table {0} not found in metadata")]
    TableNotFoundInMetadata(String),
    #[error("ERROR: failed to connect with database at {0}")]
    DBError(String),
    #[error("ERROR: Invalid input supplied. `{0}`")]
    InvalidInput(String),
}

#[derive(Error, Debug, Serialize, Clone)]
#[error("Error {{ error: `{kind}` }}")]
pub struct GQLRSError {
    pub kind: GQLRSErrorType,
}

impl GQLRSError {
    pub fn new(kind: GQLRSErrorType) -> GQLRSError {
        GQLRSError { kind }
    }
}
