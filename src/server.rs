use actix_web::http::StatusCode;
use indexmap::IndexMap;
use log::warn;
use postgres::types::Json;
use postgres::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::gql_types::{
    field_names_to_name_list, from_parser_value_to_order_by_option, is_order_by_keys_valid,
    to_int_arg, to_object_arg, to_string_arg, FieldInfo, FieldName, GQLArgType, OrderByOptions,
};
use crate::metadata::{Metadata, QualifiedTable};
use crate::{context, db, error};

pub async fn healthz_handler(
    srv_ctx: actix_web::web::Data<context::ServerCtx>,
    _req: actix_web::HttpRequest,
) -> actix_web::HttpResponse {
    actix_web::web::HttpResponse::Ok().json(srv_ctx.get_status_json())
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum MetadataRequestBody {
    TrackTable(QualifiedTable),
    UntrackTable(QualifiedTable),
    // NOTE: args will be `null` for `export_metadata`
    ExportMetadata,
    ImportMetadata(Metadata),
}

#[derive(Serialize, Debug, Clone)]
pub enum MetadataResponse<T = Metadata> {
    Success(String),
    Data(T),
    Error(error::GQLRSError),
}

impl<T> actix_web::Responder for MetadataResponse<T>
where
    T: Serialize,
{
    type Error = actix_web::Error;
    type Future = actix_web::HttpResponse;

    fn respond_to(self, _: &actix_web::HttpRequest) -> Self::Future {
        match self {
            MetadataResponse::Error(err_resp) => {
                actix_web::HttpResponse::build(StatusCode::BAD_REQUEST).json(err_resp)
            }
            MetadataResponse::Success(msg) => {
                actix_web::HttpResponse::Ok().json(json!({"success": true, "message": msg}))
            }
            MetadataResponse::Data(body) => actix_web::web::HttpResponse::Ok().json(body),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ErrorResponse {
    error: String,
}

// NOTE: this should be used for sending the API response
#[derive(Serialize, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DataResponse {
    data: IndexMap<String, serde_json::Value>,
}

impl DataResponse {
    pub fn new(data: IndexMap<String, serde_json::Value>) -> DataResponse {
        DataResponse { data }
    }
}

pub async fn metadata_handler(
    app_state: actix_web::web::Data<context::AppState>,
    payload: actix_web::web::Json<MetadataRequestBody>,
) -> impl actix_web::Responder {
    let mut server_ctx = app_state.0.lock().unwrap();

    match payload.into_inner() {
        MetadataRequestBody::TrackTable(table) => {
            match (*server_ctx).metadata_track_table(table.clone()) {
                Ok(_) => MetadataResponse::Success(format!(
                    "{} is now being tracked!",
                    table.to_string()
                )),
                Err(err) => MetadataResponse::Error(err),
            }
        }
        MetadataRequestBody::UntrackTable(table) => {
            match (*server_ctx).metadata_untrack_table(table.clone()) {
                Ok(_) => MetadataResponse::Success(format!(
                    "{} has now been un-tracked!",
                    table.to_string()
                )),
                Err(err) => MetadataResponse::Error(err),
            }
        }
        MetadataRequestBody::ExportMetadata => {
            MetadataResponse::Data((*server_ctx).get_metadata().clone())
        }
        MetadataRequestBody::ImportMetadata(md) => {
            (*server_ctx).replace_metadata(&md);
            MetadataResponse::Success("Imported metadata successfully!".to_string())
        }
    }
}

#[inline(always)]
pub fn empty_query_variables() -> IndexMap<String, String> {
    IndexMap::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(default = "empty_query_variables")]
    pub variables: IndexMap<String, String>,
}

type GQLResult = IndexMap<String, serde_json::Value>;

fn fetch_result_from_query_fields<'a>(
    qry_sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
    // NOTE: since we're not using any specific information from the query we could
    // move to using the selection set without having to duplicating code for
    // many of the patterns, like Query, Selection Set and eventually Subscriptions!
    pg_client: &mut Client,
    metadata: &Metadata,
) -> actix_web::HttpResponse {
    let mut fields_map: IndexMap<FieldName, FieldInfo> = IndexMap::new();

    for set in qry_sel_set.items.iter() {
        // NOTE: Nothing's being done for the other variants of the Selection enum
        if let graphql_parser::query::Selection::Field(field) = set {
            // FIXME: make this recursive too, this is just one level now...
            let table_name = field.name.to_string();
            let alias = field.alias.map(String::from);
            let root_field_name = FieldName::new(&table_name, alias);
            let mut field_args: IndexMap<String, GQLArgType<OrderByOptions>> = IndexMap::new();
            let sub_fields = selection_set_fields_parser(&field.selection_set);

            if !field.arguments.is_empty() {
                for root_field_arg in field.arguments.iter() {
                    let arg_name = root_field_arg.0.to_string();
                    let arg_value = &root_field_arg.1;
                    match arg_name.as_str() {
                        "order_by" => {
                            let str_field_names = field_names_to_name_list(&sub_fields);
                            if !is_order_by_keys_valid(&str_field_names, arg_value) {
                                return actix_web::HttpResponse::Ok().json(ErrorResponse{
                                    error: format!("Invalid argument values supplied to `order_by`: {}. The keys must be one off {:?} and should be used at most once", arg_value, str_field_names)
                                });
                            }
                            let convert_to_object_arg = to_object_arg(
                                arg_name.to_string(),
                                arg_value,
                                from_parser_value_to_order_by_option,
                            );
                            if let Ok(fa) = convert_to_object_arg {
                                field_args.insert(fa.0, fa.1);
                            } else if let Err(e) = convert_to_object_arg {
                                return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                    error: e.to_string(),
                                });
                            }
                        }
                        "limit" | "offset" => {
                            let convert_to_int_arg = to_int_arg(arg_name.to_string(), arg_value);
                            if let Ok(fa) = convert_to_int_arg {
                                field_args.insert(fa.0, fa.1);
                            } else if let Err(e) = convert_to_int_arg {
                                return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                    error: e.to_string(),
                                });
                            }
                        }
                        "distinct_on" => {
                            let convert_to_string_arg =
                                to_string_arg(arg_name.to_string(), arg_value);
                            match convert_to_string_arg {
                                Ok(fa) => {
                                    let str_fields = field_names_to_name_list(&sub_fields);
                                    if str_fields.contains(&fa.1.get_string()) {
                                        field_args.insert(fa.0, fa.1);
                                    } else {
                                        return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                            error: format!(
                                            "The value for `distinct_on` should be one of: {:?}",
                                            str_fields
                                        ),
                                        });
                                    }
                                }
                                Err(e) => {
                                    return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                        error: e.to_string(),
                                    });
                                }
                            }
                        }
                        _ => {
                            // NOTE: we're completely disregarding the users argument if it's none of the above
                            warn!(
                                "Arguement `{}` isn't supported and hence being ignored in the query",
                                arg_name // TODO: maybe include the query as well?
                            );
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

        let query_res = db::get_rows_gql_query(
            pg_client,
            root_field_name,
            fields_info_struct,
            metadata.to_owned(),
        );
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

    actix_web::HttpResponse::Ok().json(DataResponse::new(final_res))
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
    app_state: actix_web::web::Data<context::AppState>,
    payload: actix_web::web::Json<GraphQLRequest>,
) -> actix_web::HttpResponse {
    let server_ctx = app_state.0.lock().unwrap();
    let mut pg_client = server_ctx.get_connection_pool().get().unwrap();

    match graphql_parser::parse_query::<&str>(&payload.query) {
        // NOTE: We only execute the first query/mutation/subscription that
        // gets matched/parsed. Similar to what Hasura does
        Ok(q) => match &q.definitions[0] {
            graphql_parser::query::Definition::Fragment(_) => {
                actix_web::HttpResponse::Ok().json(ErrorResponse {
                    error: error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                        "Fragments are not yet supported!".to_string(),
                    ))
                    .to_string(),
                })
            }
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
                    fetch_result_from_query_fields(
                        &qry.selection_set,
                        &mut pg_client,
                        server_ctx.get_metadata(),
                    )
                }
                graphql_parser::query::OperationDefinition::SelectionSet(sel_set) => {
                    fetch_result_from_query_fields(
                        sel_set,
                        &mut pg_client,
                        server_ctx.get_metadata(),
                    )
                }
            },
        },
        Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
            // NOTE: this is the error when no valid AST was generated
            // by the parser or any other parser failures
            error: e.to_string(),
        }),
    }
}
