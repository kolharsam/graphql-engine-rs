use postgres::NoTls;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Error,
}

type PGPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Clone, Debug)]
pub struct ServerCtx {
    conn_pool: PGPool,
    status: Status,
}

impl ServerCtx {
    pub fn new(pg_pool: PGPool) -> ServerCtx {
        ServerCtx {
            conn_pool: pg_pool,
            status: Status::Ok,
        }
    }

    pub fn get_status_json(&self) -> serde_json::Value {
        serde_json::json!({ "status": &self.status })
    }

    fn set_status_to_errored(&mut self) {
        self.status = Status::Error
    }

    fn set_status_to_healthy(&mut self) {
        self.status = Status::Ok
    }

    pub fn get_connection_pool(&self) -> &PGPool {
        &self.conn_pool
    }
}
