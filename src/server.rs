use serde::{Deserialize, Serialize};

#[path = "./context.rs"]
mod context;
#[path = "./error.rs"]
mod error;
#[path = "./types.rs"]
mod types;

pub async fn healthz_handler(_req: actix_web::HttpRequest) -> String {
    // actix_web::HttpResponse::Ok().json(json!({"Ok": true}))
    "OK".to_string()
    // actix_web::HttpResponse::Ok().finish()
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum MetadataMessage {
    TrackTable(types::QualifiedTable),
    UntrackTable(types::QualifiedTable),
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

fn empty_query_variables() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphQLRequest {
    query: String,
    #[serde(default = "empty_query_variables")]
    variables: std::collections::HashMap<String, String>,
}

fn fetch_result_from_query_fields<'a>(
    qry: &graphql_parser::query::Query<'a, &'a str>,
) -> actix_web::HttpResponse {
    let mut fields_map: indexmap::IndexMap<String, Vec<String>> =
        indexmap::IndexMap::new();

    for set in qry.selection_set.items.iter() {
        if let graphql_parser::query::Selection::Field(field) = set {
            // FIXME: make this recursive too, this is just one level now...
            let table_name = field.name.to_string();
            let query_fields = selection_set_fields_parser(&field.selection_set);
            fields_map.insert(table_name, query_fields);
        }
    }

    // TODO: make the query to DB, fetch result, format and respond
    actix_web::HttpResponse::Ok().json(ErrorResponse {
        error: format!("{:?}", fields_map),
    })
}

fn selection_set_fields_parser<'a>(
    sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();

    // TODO: aliases are also not supported
    for set_item in sel_set.items.iter() {
        if let graphql_parser::query::Selection::Field(fld) = set_item {
            fields.push(fld.name.to_string());
        }
    }

    fields
}

fn fetch_result_from_selection_set<'a>(
    sel_set: &graphql_parser::query::SelectionSet<'a, &'a str>,
) -> actix_web::HttpResponse {
    let mut fields_map: indexmap::IndexMap<String, Vec<String>> =
        indexmap::IndexMap::new();

    for set_item in sel_set.items.iter() {
        if let graphql_parser::query::Selection::Field(fld) = set_item {
            fields_map.insert(
                fld.name.to_string(),
                selection_set_fields_parser(&fld.selection_set),
            );
        }
    }

    actix_web::HttpResponse::Ok().json(ErrorResponse {
        error: format!("{:?}", fields_map),
    })
}

// NOTE: this is only for Query, Selection sets Mutation(which are currently not supported)
//       will have to implement something different for Subscriptions.
pub async fn graphql_handler(payload: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let parse_result = json::parse(std::str::from_utf8(&payload).unwrap());

    // FIXME?: is this even necessary?
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

    match serde_json::from_str::<'_, GraphQLRequest>(&body.dump()) {
        Ok(b) => {
            match graphql_parser::parse_query::<&str>(&b.query) {
                Ok(q) => {
                    for i in q.definitions.iter() {
                        match i {
                            graphql_parser::query::Definition::Fragment(_) => {
                                return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                    error: error::GQLRSError {
                                        kind: error::GQLRSErrorType::GenericError(
                                            "Fragments are not yet supported!".to_string(),
                                        ),
                                    }
                                    .to_string(),
                                })
                            }
                            graphql_parser::query::Definition::Operation(op) => match op {
                                graphql_parser::query::OperationDefinition::Mutation(_) => {
                                    return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                        error: error::GQLRSError {
                                            kind: error::GQLRSErrorType::GenericError(
                                                "Mutations are not yet supported!".to_string(),
                                            ),
                                        }
                                        .to_string(),
                                    })
                                }
                                graphql_parser::query::OperationDefinition::Subscription(_) => {
                                    return actix_web::HttpResponse::Ok().json(ErrorResponse {
                                        error: error::GQLRSError {
                                            kind: error::GQLRSErrorType::GenericError(
                                                "Subscriptions are not yet supported!".to_string(),
                                            ),
                                        }
                                        .to_string(),
                                    })
                                }
                                graphql_parser::query::OperationDefinition::Query(qry) => {
                                    return fetch_result_from_query_fields(qry)
                                }
                                graphql_parser::query::OperationDefinition::SelectionSet(
                                    sel_set,
                                ) => return fetch_result_from_selection_set(sel_set),
                            },
                        }
                    }
                }
                Err(e) => {
                    return actix_web::HttpResponse::Ok().json(ErrorResponse {
                        error: e.to_string(),
                    })
                }
            }
            actix_web::HttpResponse::Ok().json(error::GQLRSError {
                kind: error::GQLRSErrorType::GenericError("Unsolicited error".to_string()),
            })
        }
        Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}
