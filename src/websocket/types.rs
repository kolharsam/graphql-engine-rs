use actix::{Message as ActixMessage, Recipient};
use actix_web_actors::ws::WebsocketContext;
use indexmap::IndexMap;
use json::iterators::Members;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

use crate::error::GQLRSError;

use super::WebSocketSession;

pub const GRAPHQL_TRANSPORT_WS_PROTOCOL: &str = "graphql-transport-ws";

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Message(pub String);

pub trait ToMessage {
    fn to_message(&self) -> Result<Message, serde_json::Error>;
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
// TODO: find a better name for this struct
pub struct ClientPayload {
    pub id: String,
    pub message: ClientMessage,
}

impl ClientPayload {
    pub fn new(id: String, message: ClientMessage) -> Self {
        Self { id, message }
    }
}

#[derive(ActixMessage, Deserialize, Debug)]
#[rtype(result = "()")]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    ConnectionInit {
        payload: Option<HashMap<String, String>>,
    },
    Complete {
        id: String,
    },
    Subscribe {
        payload: MessagePayload,
        id: String,
    },
    Ping {
        payload: Option<HashMap<String, String>>,
    },
    Pong {
        payload: Option<HashMap<String, String>>,
    },
    Invalid(String),
}

impl From<Message> for ClientMessage {
    fn from(message: Message) -> Self {
        ClientMessage::from_str(&message.0).unwrap_or(ClientMessage::Invalid(message.0))
    }
}

impl FromStr for ClientMessage {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<ClientMessage>(s)
    }
}

#[derive(ActixMessage, Serialize, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
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
    Next {
        payload: ExecutionResult,
        id: String,
    },
    Ping {
        payload: Option<HashMap<String, String>>,
    },
    Pong {
        payload: Option<HashMap<String, String>>,
    },
}

impl From<ServerMessage> for Message {
    fn from(sm: ServerMessage) -> Self {
        Message(sm.to_string())
    }
}
impl ToString for ServerMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or("".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePayload {
    extensions: Option<HashMap<String, String>>,
    #[serde(rename = "camelCase")]
    operation_name: Option<String>,
    query: String,
    variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Clone)]
pub enum ExecutionResult {
    Data(IndexMap<String, serde_json::Value>),
    Errors(Vec<GQLRSError>),
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub id: String,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}
