use indexmap::IndexMap;
use postgres::types::Json;
use postgres::{Client, Row};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::error;
use crate::types::{
    field_names_to_name_list, to_int_arg, to_string_arg, FieldInfo, FieldName, GQLArgType,
    QualifiedTable, SUPPORTED_INT_GQL_ARGUMENTS, SUPPORTED_STRING_GQL_ARGUMENTS,
};

pub async fn healthz_handler(_req: actix_web::HttpRequest) -> String {
    "OK".to_string()
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum MetadataMessage {
    TrackTable(QualifiedTable),
    UntrackTable(QualifiedTable),
    // NOTE: args will only be `null` in this case
    ExportMetadata,
}

#[derive(Serialize)]
pub struct MetadataSuccess {
    success: bool,
    message: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ErrorResponse {
    error: String,
}

// NOTE: this should be used for sending the API response
#[derive(Serialize, Debug, Clone)]
pub struct DataResponse {
    data: IndexMap<String, serde_json::Value>,
}

pub async fn metadata_handler(payload: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let payload_to_str = std::str::from_utf8(&payload);
    let parse_result = match payload_to_str {
        Ok(r) => json::parse(r),
        Err(_err) => Err(json::Error::FailedUtf8Parsing),
    };

    let body = parse_result.unwrap_or_else(|e| json::object! { "error" => e.to_string() });

    match serde_json::from_str::<'_, MetadataMessage>(&body.dump()) {
        Ok(b) => actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(
                serde_json::to_string(&b)
                    .unwrap_or_else(|_| "Failed to convert body to a valid string".to_string()),
            ),
        Err(e) => actix_web::HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST).json(
            ErrorResponse {
                error: e.to_string(),
            },
        ),
    }

    // MetadataResponse::Success(MetadataSuccess {
    //     success: true,
    //     message: "OK".to_string(),
    // })
}

fn empty_query_variables() -> IndexMap<String, String> {
    IndexMap::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphQLRequest {
    query: String,
    #[serde(default = "empty_query_variables")]
    variables: IndexMap<String, String>,
}

type GQLResult = IndexMap<String, serde_json::Value>;

fn fetch_result_from_query_fields<'a>(
    qry_sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
    // NOTE: since we're not using any specific information from the query we could
    // move to using the selection set without having to duplicating code for
    // many of the patterns, like Query, Selection Set and eventually Subscriptions!
    pg_client: &mut Client,
) -> actix_web::HttpResponse {
    let mut fields_map: IndexMap<FieldName, FieldInfo> = IndexMap::new();

    for set in qry_sel_set.items.iter() {
        // NOTE: Nothing's being done for the other variants of the Selection enum
        if let graphql_parser::query::Selection::Field(field) = set {
            // FIXME: make this recursive too, this is just one level now...
            let table_name = field.name.to_string();
            let alias = field.alias.map(String::from);
            let root_field_name = FieldName::new(&table_name, alias);
            let mut field_args: IndexMap<String, GQLArgType> = IndexMap::new();
            let sub_fields = selection_set_fields_parser(&field.selection_set);

            if !field.arguments.is_empty() {
                for root_field_arg in field.arguments.iter() {
                    let arg_name = root_field_arg.0.to_string();
                    let arg_value = &root_field_arg.1;
                    if SUPPORTED_STRING_GQL_ARGUMENTS.contains(&arg_name.as_str()) {
                        let convert_to_string_arg = to_string_arg(arg_name, arg_value);
                        if let Ok(fa) = convert_to_string_arg {
                            let str_fields = field_names_to_name_list(&sub_fields);
                            if str_fields.contains(&fa.1.get_string()) {
                                field_args.insert(fa.0, fa.1);
                            } else {
                                return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                    error: format!("The value should be one of: {:?}", str_fields),
                                });
                            }
                        } else if let Err(e) = convert_to_string_arg {
                            return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                error: e.to_string(),
                            });
                        }
                    } else if SUPPORTED_INT_GQL_ARGUMENTS.contains(&arg_name.as_str()) {
                        let convert_to_int_arg = to_int_arg(arg_name, arg_value);
                        if let Ok(fa) = convert_to_int_arg {
                            field_args.insert(fa.0, fa.1);
                        } else if let Err(e) = convert_to_int_arg {
                            return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }

            fields_map.insert(root_field_name, FieldInfo::new(sub_fields, field_args));
        }
    }

    let mut result_rows: Vec<Row> = Vec::new();
    for field_info in fields_map.iter() {
        let root_field_name = field_info.0;
        let fields_info_struct = field_info.1;

        let query_res = db::get_rows_gql_query(pg_client, root_field_name, fields_info_struct);
        match query_res {
            Ok(db_res) => {
                result_rows.push(db_res);
            }
            // NOTE: this error is encounted when the query fails at the DB
            Err(db_err) => {
                return actix_web::HttpResponse::Ok().json(ErrorResponse {
                    error: db_err.to_string(),
                })
            }
        }
    }

    let mut final_res: GQLResult = IndexMap::new();

    for res_row in result_rows.iter() {
        // FIXME: can be a potential point of failure
        let root_field_name = res_row.columns()[0].name();
        let query_result: Result<Json<serde_json::Value>, postgres::Error> =
            res_row.try_get(root_field_name);
        match query_result {
            Ok(result) => {
                final_res.insert(root_field_name.to_string(), result.0);
            }
            // NOTE: this error is reported when we encounter no rows or nulls
            Err(_err) => {
                final_res.insert(
                    root_field_name.to_string(),
                    serde_json::json!({ root_field_name.to_string(): serde_json::Value::Null }),
                );
            }
        }
    }

    actix_web::HttpResponse::Ok().json(DataResponse { data: final_res })
}

fn selection_set_fields_parser<'a>(
    sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
) -> Vec<FieldName> {
    let mut fields: Vec<FieldName> = Vec::new();

    for set_item in sel_set.items.iter() {
        if let graphql_parser::query::Selection::Field(fld) = set_item {
            let alias = fld.alias.map(String::from);
            fields.push(FieldName::new(fld.name, alias));
        }
    }

    fields
}

// NOTE: Only GraphQL Queries and Selection Sets are supported.
//       Mutations, Subscriptions will be supported eventually.
pub async fn graphql_handler(
    srv_ctx: actix_web::web::Data<&'static str>,
    payload: actix_web::web::Bytes,
) -> actix_web::HttpResponse {
    let conn_str = srv_ctx.get_ref();
    let mut pg_client = db::get_pg_client(conn_str.to_string());

    let payload_to_str = std::str::from_utf8(&payload);

    let parse_result = match payload_to_str {
        Ok(s) => json::parse(s),
        Err(_err) => Err(json::Error::FailedUtf8Parsing),
    };

    let body = parse_result.unwrap_or_else(|e| json::object! { "error" => e.to_string() });

    match serde_json::from_str::<'_, GraphQLRequest>(&body.dump()) {
        Ok(b) => match graphql_parser::parse_query::<&str>(&b.query) {
            // NOTE: We only execute the first query/mutation/subscription that
            // gets matched/parsed. Similar to what Hasura does
            Ok(q) => match &q.definitions[0] {
                graphql_parser::query::Definition::Fragment(_) => actix_web::HttpResponse::Ok()
                    .json(ErrorResponse {
                        error: error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                            "Fragments are not yet supported!".to_string(),
                        ))
                        .to_string(),
                    }),
                graphql_parser::query::Definition::Operation(op) => match op {
                    graphql_parser::query::OperationDefinition::Mutation(_) => {
                        actix_web::HttpResponse::Ok().json(ErrorResponse {
                            error: error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                                "Mutations are not yet supported!".to_string(),
                            ))
                            .to_string(),
                        })
                    }
                    graphql_parser::query::OperationDefinition::Subscription(_) => {
                        actix_web::HttpResponse::Ok().json(ErrorResponse {
                            error: error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                                "Subscriptions are not yet supported!".to_string(),
                            ))
                            .to_string(),
                        })
                    }
                    graphql_parser::query::OperationDefinition::Query(qry) => {
                        fetch_result_from_query_fields(&qry.selection_set, &mut pg_client)
                    }
                    graphql_parser::query::OperationDefinition::SelectionSet(sel_set) => {
                        fetch_result_from_query_fields(sel_set, &mut pg_client)
                    }
                },
            },
            Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
                // NOTE: this is the error when no valid AST was generated
                // by the parser or any other parser failures
                error: e.to_string(),
            }),
        },
        Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
            // NOTE: this is the error when the parsed JSON is
            // not of the type of GraphQLRequest
            error: e.to_string(),
        }),
    }
}
