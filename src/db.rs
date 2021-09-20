use postgres::{Client, NoTls, Row};
use r2d2::{Error, Pool};
use r2d2_postgres::PostgresConnectionManager;

use crate::error;
use crate::gql_types::{FieldInfo, FieldName, GQLArgTypeWithOrderBy, SUPPORTED_INT_GQL_ARGUMENTS};
use crate::utils;

pub fn get_pg_pool(
    connection_string: &str,
) -> Result<Pool<PostgresConnectionManager<NoTls>>, Error> {
    // NOTE: NoTls is only for the time being, we might have to support TLS based connections eventually
    let manager = PostgresConnectionManager::new(connection_string.parse().unwrap(), NoTls);
    Pool::new(manager)
}

#[inline]
fn add_int_arg_to_query(
    query_str: &mut String,
    arg_name: &str,
    arg_value: Option<&GQLArgTypeWithOrderBy>,
) {
    match arg_value {
        None => (),
        Some(val) => {
            query_str.push_str(format!("{} {} ", arg_name.to_uppercase(), val.get_num()).as_str());
        }
    }
}

// This is a helper to construct the SQL query to fetch results from the database
pub fn get_rows_gql_query(
    client: &mut Client,
    root_field: &FieldName,
    field_info: &FieldInfo,
) -> Result<Row, error::GQLRSError> {
    let mut query = String::new();
    let query_has_args = !field_info.args().is_empty();

    // NOTE: since we're using json_agg here, the DB has to be of v9 or over
    query.push_str(
        format!(
            "SELECT coalesce(json_agg(data), '[]') AS {} FROM (SELECT ",
            root_field.alias()
        )
        .as_str(),
    );

    // add the distinct on clause (if necessary)
    if query_has_args && field_info.args().contains_key("distinct_on") {
        let distinct_col = field_info.args().get("distinct_on");
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

    for field_name in field_info.fields().iter() {
        query.push_str(format!("{}, ", field_name.to_sql()).as_str());
    }

    // remove the extra (", ")
    query.pop();
    query.pop();

    // FIXME/TODO: support other schemas based on the info that might be stored in metadata
    query.push_str(format!(" FROM \"public\".\"{}\" ", root_field.name()).as_str());

    if query_has_args {
        SUPPORTED_INT_GQL_ARGUMENTS.iter().for_each(|field_arg| {
            let arg_val = field_info.args().get(*field_arg);
            match *field_arg {
                "limit" => {
                    add_int_arg_to_query(&mut query, "limit", arg_val);
                }
                "offset" => {
                    add_int_arg_to_query(&mut query, "offset", arg_val);
                }
                _ => (),
            }
        });
    }

    // See if there's a requirement of the `order by` clause
    if query_has_args && field_info.args().contains_key("order_by") {
        let order_by_cols = field_info.args().get("order_by");
        match order_by_cols {
            Some(val) => {
                query.push_str(" ORDER BY ");
                let order_by_map = val.get_object();

                for (col_name, order_by_clause) in order_by_map.iter() {
                    let quoted_col_name = utils::dquote(col_name);
                    query.push_str(
                        format!("{} {},", quoted_col_name, order_by_clause.to_sql()).as_str(),
                    );
                }

                // NOTE: Popping the last character here for the hanging comma that
                // might be present upon adding these statements to the query string
                query.pop();
            }
            // NOTE: this is not plausible
            None => {
                return Err(error::GQLRSError::new(error::GQLRSErrorType::InvalidInput(
                    "argument value not found".to_string(),
                )));
            }
        }
    }

    query.push_str(") as data");

    // ----- Query construction ends

    // ----- Run Query

    let query_result = client.query_one(query.as_str(), &[]);

    query_result
        .map_err(|err| error::GQLRSError::new(error::GQLRSErrorType::DBError(format!("{:?}", err))))
}
