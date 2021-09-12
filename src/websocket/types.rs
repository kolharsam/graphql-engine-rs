use actix::{Message as ActixMessage, Recipient};
use actix_web_actors::ws::{CloseCode, CloseReason};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

use crate::error::GQLRSError;

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

#[derive(ActixMessage, Serialize)]
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

#[derive(Debug, Serialize)]
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

pub enum GQLCloseCode {
    InternalServerError,
    BadRequest,
    /** Tried subscribing before connect ack */
    Unauthorized,
    Forbidden,
    SubprotocolNotAcceptable,
    ConnectionInitialisationTimeout,
    ConnectionAcknowledgementTimeout,
    TooManyInitialisationRequests,
    SubscriberAlreadyExists(String),
}

impl GQLCloseCode {
    fn to_close_reason(&self, description: &str) -> CloseReason {
        CloseReason {
            code: self.into(),
            description: Some(description.to_string()),
        }
    }
}
impl From<&GQLCloseCode> for CloseCode {
    fn from(code: &GQLCloseCode) -> Self {
        match code {
            GQLCloseCode::InternalServerError => CloseCode::Other(4500),
            GQLCloseCode::BadRequest => CloseCode::Other(4400),
            GQLCloseCode::Unauthorized => CloseCode::Other(4401),
            GQLCloseCode::Forbidden => CloseCode::Other(4403),
            GQLCloseCode::SubprotocolNotAcceptable => CloseCode::Other(4406),
            GQLCloseCode::ConnectionInitialisationTimeout => CloseCode::Other(4408),
            GQLCloseCode::ConnectionAcknowledgementTimeout => CloseCode::Other(4504),
            GQLCloseCode::TooManyInitialisationRequests => CloseCode::Other(4429),
            GQLCloseCode::SubscriberAlreadyExists(_) => CloseCode::Other(4409),
        }
    }
}
impl From<&GQLCloseCode> for CloseReason {
    fn from(gql_code: &GQLCloseCode) -> Self {
        match gql_code {
            GQLCloseCode::InternalServerError => gql_code.to_close_reason("Internal server error"),
            GQLCloseCode::BadRequest => gql_code.to_close_reason("Bad request"),
            GQLCloseCode::Unauthorized => gql_code.to_close_reason("Unathorized"),
            GQLCloseCode::Forbidden => gql_code.to_close_reason("Forbidden"),
            GQLCloseCode::SubprotocolNotAcceptable => {
                gql_code.to_close_reason("Subprotocol not acceptable")
            }
            GQLCloseCode::ConnectionInitialisationTimeout => {
                gql_code.to_close_reason("Connection initialisation timeout")
            }
            GQLCloseCode::ConnectionAcknowledgementTimeout => {
                gql_code.to_close_reason("Connection acknowledgement timeout")
            }
            GQLCloseCode::TooManyInitialisationRequests => {
                gql_code.to_close_reason("Too many initialisation requests")
            }
            GQLCloseCode::SubscriberAlreadyExists(ref id) => {
                gql_code.to_close_reason(&format!("Subscriber for {} already exists", id))
            }
        }
    }
}
impl From<GQLCloseCode> for Option<CloseReason> {
    fn from(code: GQLCloseCode) -> Self {
        Some((&code).into())
    }
}
