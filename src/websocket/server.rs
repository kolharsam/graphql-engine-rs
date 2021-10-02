use super::types::{
    ClientMessage, ClientPayload, Connect, Disconnect, Message, MessagePayload, ServerMessage,
};
use actix::{Actor, Context, Handler, Recipient};
use indexmap::IndexMap;
use log::error;
use serde_json::error::Result as SerdeResult;

pub struct WebSocketServer {
    sessions: IndexMap<String, Recipient<Message>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            sessions: IndexMap::new(),
        }
    }

    fn create_trigger_sql(id: &str, table: &str) -> String {
        let trigger_id = id.replace("-", "");
        format!("public.create_trigger({}, {});", trigger_id, table)
    }

    fn drop_trigger_sql(id: &str, table: &str) -> String {
        let trigger_id = id.replace("-", "");
        format!("DROP TRIGGER IF EXISTS {} ON {}", trigger_id, table)
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
    fn handle_subscription(&self, payload: &MessagePayload, id: &str) {
        println!("Message from client: {:?}", payload)
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

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) {
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
            ClientMessage::Subscribe { payload, id } => self.handle_subscription(payload, id),
            ClientMessage::Ping { payload: _ } => self.handle_ping(client_payload),
            ClientMessage::Pong { payload: _ } => self.handle_pong(client_payload),
            ClientMessage::Invalid(text) => error!("Message from client: {:?}", text),
        }
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}
