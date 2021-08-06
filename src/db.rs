use postgres::{Client, NoTls};
#[path = "./error.rs"]
mod error;

pub fn connect_db(connection_string: String) -> Result<postgres::Client, error::GQLRSError> {
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Ok(c) => Ok(c),
        Err(err) => Err(error::GQLRSError::new(
            error::GQLRSErrorType::DBConnectionError(err.to_string()),
        )),
    }
}
