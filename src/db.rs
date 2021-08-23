use crate::error;
use crate::types;
use postgres::{Client, NoTls, Row};

pub fn get_pg_client(connection_string: String) -> Client {
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Ok(c) => c,
        Err(e) => panic!(
            "{}",
            error::GQLRSError::new(error::GQLRSErrorType::DBConnectionError(e.to_string()),)
        ),
    }
}

pub fn get_rows_gql_query(
    client: &mut Client,
    root_field: &types::FieldName,
    fields: &[types::FieldName],
) -> Result<Row, postgres::error::Error> {
    let mut query = String::new();
    // NOTE: since we're using json_agg here, the DB has to be of v9 or over
    query.push_str(
        format!(
            "SELECT json_agg(data) AS {} FROM (SELECT ",
            root_field.alias()
        )
        .as_str(),
    );

    for field_name in fields.iter() {
        query.push_str(format!("{}, ", field_name.to_sql()).as_str());
    }

    // remove the extra (", ")
    query.pop();
    query.pop();

    // FIXME?: support other schemas based on the info that might be stored in metadata
    query.push_str(format!(" FROM \"public\".\"{}\") as data", root_field.name()).as_str());

    // ----- Query construction ends

    client.query_one(query.as_str(), &[])
}
