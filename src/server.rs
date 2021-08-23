use postgres::types::Json;
use postgres::{Client, Row};
use serde::{Deserialize, Serialize};

// use crate::context;
use crate::db;
use crate::error;
use crate::types::{FieldName, QualifiedTable};
use crate::utils;

pub async fn healthz_handler(_req: actix_web::HttpRequest) -> String {
    // actix_web::HttpResponse::Ok().json(json!({"Ok": true}))
    "OK".to_string()
    // actix_web::HttpResponse::Ok().finish()
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
    data: indexmap::IndexMap<String, serde_json::Value>,
}

pub async fn metadata_handler(payload: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let parse_result = json::parse(std::str::from_utf8(&payload).unwrap());

    // FIXME?: is this even necessary?
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

    match serde_json::from_str::<'_, MetadataMessage>(&body.dump()) {
        Ok(b) => actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&b).unwrap()),
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

fn empty_query_variables() -> indexmap::IndexMap<String, String> {
    indexmap::IndexMap::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphQLRequest {
    query: String,
    #[serde(default = "empty_query_variables")]
    variables: indexmap::IndexMap<String, String>,
}

type GQLResult = indexmap::IndexMap<String, serde_json::Value>;

fn fetch_result_from_query_fields<'a>(
    qry: &graphql_parser::query::Query<'a, &'a str>,
    client: &mut Client,
) -> actix_web::HttpResponse {
    let mut fields_map: indexmap::IndexMap<FieldName, Vec<FieldName>> = indexmap::IndexMap::new();

    for set in qry.selection_set.items.iter() {
        if let graphql_parser::query::Selection::Field(field) = set {
            // FIXME: make this recursive too, this is just one level now...
            let table_name = field.name.to_string();
            let alias = field.alias.map(String::from);
            let root_field_name = FieldName::new(&table_name, alias);
            fields_map.insert(
                root_field_name,
                selection_set_fields_parser(&field.selection_set),
            );
        }
    }

    let mut result_rows: Vec<Row> = Vec::new();
    for field_info in fields_map.iter() {
        let query_res = db::get_rows_gql_query(client, field_info.0, field_info.1);
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

    let mut final_res: GQLResult = indexmap::IndexMap::new();

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
                // NOTE: this is not consistent with Hasura's behavior, in Hasura,
                // all of the columns in the query are placed with a `null`
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

fn fetch_result_from_selection_set<'a>(
    sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
    _client: &mut Client,
) -> actix_web::HttpResponse {
    let mut fields_map: indexmap::IndexMap<FieldName, Vec<FieldName>> = indexmap::IndexMap::new();

    for set_item in sel_set.items.iter() {
        if let graphql_parser::query::Selection::Field(fld) = set_item {
           let alias = fld.alias.map(String::from);
            let root_field_name = FieldName::new(fld.name, alias);
            fields_map.insert(
                root_field_name,
                selection_set_fields_parser(&fld.selection_set),
            );
        }
    }

    // TODO: this shouldn't be here anymore :P
    actix_web::HttpResponse::Ok().json(ErrorResponse {
        error: format!("{:?}", fields_map),
    })
}

// NOTE: Only GraphQL Queries and Selection Sets are supported.
//       Mutations, Subscriptions will be supported eventually.
pub async fn graphql_handler(
    srv_ctx: actix_web::web::Data<&'static str>,
    payload: actix_web::web::Bytes,
) -> actix_web::HttpResponse {
    let conn_str = srv_ctx.get_ref();
    let mut pg_client = db::get_pg_client(conn_str.to_string());

    let parse_result = json::parse(std::str::from_utf8(&payload).unwrap());

    // FIXME?: is this even necessary?
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

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
                        fetch_result_from_query_fields(qry, &mut pg_client)
                    }
                    graphql_parser::query::OperationDefinition::SelectionSet(sel_set) => {
                        fetch_result_from_selection_set(sel_set, &mut pg_client)
                    }
                },
            },
            Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
                error: e.to_string(),
            }),
        },
        Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}
