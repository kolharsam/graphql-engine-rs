use thiserror::Error;

#[derive(Error, Debug, serde::Serialize, Clone)]
pub enum GQLRSErrorType {
    #[error("ERROR: `{0}`")]
    GenericError(String),
    #[error("ERROR: Table `{0}` is already tracked")]
    TableAlreadyTracked(String),
    #[error("ERROR: Table {0} not found in metadata")]
    TableNotFoundInMetadata(String),
    #[error("Error: failed to connect with database at {0}")]
    DBError(String),
    #[error("ERROR: Invalid input was supplied.")]
    InvalidInput,
}

#[derive(Error, Debug, serde::Serialize, Clone)]
#[error("Error {{ kind: `{kind}` }}")]
pub struct GQLRSError {
    pub kind: GQLRSErrorType,
}

impl GQLRSError {
    pub fn new(kind: GQLRSErrorType) -> GQLRSError {
        GQLRSError { kind }
    }
}
