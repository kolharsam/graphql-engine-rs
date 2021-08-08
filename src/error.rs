#[derive(Debug)]
pub enum GQLRSErrorType {
    GenericError(String),
    TableAlreadyTracked(String),
    TableNotFoundInMetadata(String),
    DBConnectionError(String),
    InvalidInput,
}

impl std::fmt::Display for GQLRSErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GQLRSErrorType::GenericError(msg) => write!(f, "ERROR: {}", msg),
            GQLRSErrorType::TableAlreadyTracked(qualified_table_str) => {
                write!(f, "ERROR: Table {} is already tracked", qualified_table_str)
            }
            GQLRSErrorType::TableNotFoundInMetadata(qualified_table_str) => {
                write!(
                    f,
                    "ERROR: Table {} not found in metadata",
                    qualified_table_str
                )
            }
            GQLRSErrorType::DBConnectionError(connection_string) => write!(
                f,
                "Error: failed to connect with database at {}",
                connection_string
            ),
            GQLRSErrorType::InvalidInput => write!(f, "ERROR: Invalid input was supplied."),
        }
    }
}

impl std::error::Error for GQLRSErrorType {}

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
