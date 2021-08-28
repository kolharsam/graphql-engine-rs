use crate::error;
use crate::types;
use postgres::{Client, NoTls, Row};

pub fn get_pg_client(connection_string: String) -> Client {
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Ok(c) => c,
        // NOTE: Panic-ing since this is a crisis, the DB is out,
        // should move to a more safe handling of this sooner than later
        Err(e) => panic!(
            "{}",
            error::GQLRSError::new(error::GQLRSErrorType::DBError(e.to_string()))
        ),
    }
}

pub fn get_rows_gql_query(
    client: &mut Client,
    root_field: &types::FieldName,
    field_info: &types::FieldInfo,
) -> Result<Row, error::GQLRSError> {
    let mut query = String::new();
    let query_has_args = !field_info.root_field_arguments.is_empty();

    // NOTE: since we're using json_agg here, the DB has to be of v9 or over
    query.push_str(
        format!(
            "SELECT coalesce(json_agg(data), '[]') AS {} FROM (SELECT ",
            root_field.alias()
        )
        .as_str(),
    );

    // add the distinct on clause (if necessary)
    if query_has_args && field_info.root_field_arguments.contains_key("distinct_on") {
        let distinct_col = field_info.root_field_arguments.get("distinct_on");
        match distinct_col {
            Some(val) => {
                query.push_str(format!("DISTINCT ON({}) ", val.get_string()).as_str());
            }
            None => {
                // NOTE: this case would be highly unlikely since we're checking whether
                // the key is present at all in the first place
                return Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                    "argument value not found".to_string(),
                )));
            }
        }
    }

    for field_name in field_info.fields.iter() {
        query.push_str(format!("{}, ", field_name.to_sql()).as_str());
    }

    // remove the extra (", ")
    query.pop();
    query.pop();

    // FIXME/TODO: support other schemas based on the info that might be stored in metadata
    query.push_str(format!(" FROM \"public\".\"{}\" ", root_field.name()).as_str());

    if query_has_args {
        for field_arg in types::SUPPORTED_INT_GQL_ARGUMENTS.iter() {
            if (*field_arg) == "limit" {
                let arg_val = field_info.root_field_arguments.get("limit");
                match arg_val {
                    None => {}
                    Some(val) => {
                        query.push_str(format!("LIMIT {} ", val.get_num()).as_str());
                    }
                }
            }

            if (*field_arg) == "offset" {
                let arg_val = field_info.root_field_arguments.get("offset");
                match arg_val {
                    None => {}
                    Some(val) => {
                        query.push_str(format!("OFFSET {} ", val.get_num()).as_str());
                    }
                }
            }
        }
    }

    query.push_str(") as data");

    // ----- Query construction ends

    // ----- Run Query

    let query_result = client.query_one(query.as_str(), &[]);

    match query_result {
        Ok(res) => Ok(res),
        Err(err) => Err(error::GQLRSError::new(error::GQLRSErrorType::DBError(
            format!("{:?}", err),
        ))),
    }
}
