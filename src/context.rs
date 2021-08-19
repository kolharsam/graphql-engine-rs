use postgres::Client;

use crate::types;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Status {
    Healthy,
    Errored,
}

pub struct ServerCtx {
    pub client: Client,
    pub metadata: types::Metadata,
    pub status: Status,
}

impl ServerCtx {
    fn new(&self, pg_client: Client, metadata: types::Metadata) -> ServerCtx {
        ServerCtx {
            client: pg_client,
            metadata,
            status: Status::Healthy,
        }
    }

    fn get_status(&self) -> Status {
        self.status
    }

    fn set_status(&mut self, status: Status) {
        self.status = status
    }

    fn get_metadata(&self) -> &types::Metadata {
        &self.metadata
    }

    // fn set_metadata(&mut self, metadata: Metadata)
    // TODO: -- This is not needed since metadata has all the methods
    // but something to think about in terms of design

    fn get_client(&self) -> &Client {
        &self.client
    }
}
