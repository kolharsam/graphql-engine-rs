use super::types::{ClientMessage, ClientPayload, Connect, Disconnect, Message, ServerMessage};
use actix::{Actor, Context, Handler, Recipient};
use log::error;
use serde_json::error::Result as SerdeResult;
use std::{collections::HashMap, time::Instant};

pub struct WebSocketServer {
    sessions: HashMap<String, Recipient<Message>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn send_message(&self, data: SerdeResult<String>) {
        match data {
            Ok(data) => {
                for recipient in self.sessions.values() {
                    match recipient.do_send(Message(data.clone())) {
                        Err(err) => {
                            error!("Error sending client message: {:?}", err);
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                error!("Data did not convert to string {:?}", err);
            }
        }
    }

    fn send_to_client(&self, client_id: &str, msg: ServerMessage) {
        if let Some(recipient) = self.sessions.get(client_id) {
            // TODO: handle errors
            let _ = recipient
                .do_send(msg.into())
                .map_err(|err| error!("Error sending client message: {:?}", err));
        }
    }

    fn handle_connection_init(&self, client_payload: ClientPayload) {
        self.send_to_client(
            &client_payload.id,
            ServerMessage::ConnectionAck { payload: None },
        );
    }

    fn handle_pong(&self, client_payload: ClientPayload) {
        self.send_to_client(&client_payload.id, ServerMessage::Ping { payload: None })
    }

    fn handle_ping(&self, client_payload: ClientPayload) {
        self.send_to_client(&client_payload.id, ServerMessage::Pong { payload: None })
    }
}

impl Handler<Connect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) {
        self.sessions.insert(msg.id.clone(), msg.addr);
    }
}

impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<ServerMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_message(serde_json::to_string(&msg));
    }
}

impl Handler<ClientPayload> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, client_payload: ClientPayload, _: &mut Context<Self>) -> Self::Result {
        match &client_payload.message {
            ClientMessage::ConnectionInit { payload: _ } => {
                self.handle_connection_init(client_payload)
            }
            ClientMessage::Complete { id } => println!("Message from client: {:?}", id),
            ClientMessage::Subscribe { payload: _, id: _ } => {
                println!("Message from client: {:?}", &client_payload.message)
            }
            ClientMessage::Ping { payload: _ } => self.handle_ping(client_payload),
            ClientMessage::Pong { payload: _ } => self.handle_pong(client_payload),
            ClientMessage::Invalid(text) => error!("Message from client: {:?}", text),
        }
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}
