use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const GRAPHQL_TRANSPORT_WS_PROTOCOL: &str = "graphql-transport-ws";

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    ConnectionInit {
        payload: Option<HashMap<String, String>>,
    },
    ConnectionAck {
        payload: Option<HashMap<String, String>>,
    },
    Error {
        payload: HashMap<String, String>,
        id: String,
    },
    Complete {
        id: String,
    },
    Subscribe {
        payload: MessagePayload,
        id: String,
    },
    Next {
        payload: HashMap<String, String>,
        id: String,
    },
    Ping {
        payload: Option<HashMap<String, String>>,
    },
    Pong {
        payload: Option<HashMap<String, String>>,
    },
    #[serde(rename = "ka")]
    KeepAlive,
}

#[derive(Serialize, Deserialize)]
pub struct MessagePayload {
    extensions: Option<HashMap<String, String>>,
    #[serde(rename = "camelCase")]
    operation_name: Option<String>,
    query: String,
    variables: Option<HashMap<String, String>>,
}
