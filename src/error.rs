use thiserror::Error;

#[derive(Error, Debug, serde::Serialize)]
pub enum GQLRSErrorType {
    #[error("ERROR: `{0}`")]
    GenericError(String),
    #[error("ERROR: Table `{0}` is already tracked")]
    TableAlreadyTracked(String),
    #[error("ERROR: Table {0} not found in metadata")]
    TableNotFoundInMetadata(String),
    #[error("Error: failed to connect with database at {0}")]
    DBConnectionError(String),
    #[error("ERROR: Invalid input was supplied.")]
    InvalidInput,
}

// impl std::fmt::Display for GQLRSErrorType {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             GQLRSErrorType::GenericError(msg) => write!(f, "ERROR: {}", msg),
//             GQLRSErrorType::TableAlreadyTracked(qualified_table_str) => {
//                 write!(f, "ERROR: Table {} is already tracked", qualified_table_str)
//             }
//             GQLRSErrorType::TableNotFoundInMetadata(qualified_table_str) => {
//                 write!(
//                     f,
//                     "ERROR: Table {} not found in metadata",
//                     qualified_table_str
//                 )
//             }
//             GQLRSErrorType::DBConnectionError(connection_string) => write!(
//                 f,
//                 "Error: failed to connect with database at {}",
//                 connection_string
//             ),
//             GQLRSErrorType::InvalidInput => write!(f, "ERROR: Invalid input was supplied."),
//         }
//     }
// }

// impl std::error::Error for GQLRSErrorType {}

#[derive(serde::Serialize)]
pub struct GQLRSError {
    pub kind: GQLRSErrorType,
}

impl GQLRSError {
    pub fn new(kind: GQLRSErrorType) -> GQLRSError {
        GQLRSError { kind }
    }
}

impl std::fmt::Debug for GQLRSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ERROR").field("kind", &self.kind).finish()
    }
}

impl std::fmt::Display for GQLRSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl std::error::Error for GQLRSError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}
