use postgres::NoTls;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

use crate::metadata::{load_metadata, Metadata, MetadataResult, QualifiedTable};

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Error,
}

type PGPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Clone, Debug)]
pub struct ServerCtx {
    conn_pool: PGPool,
    metadata: Metadata,
    status: Status,
}

impl ServerCtx {
    pub fn new(pg_pool: PGPool, source_name: String) -> ServerCtx {
        ServerCtx {
            conn_pool: pg_pool,
            status: Status::Ok,
            metadata: load_metadata(source_name),
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

    pub fn metadata_track_table(&mut self, table_info: QualifiedTable) -> MetadataResult {
        self.metadata.track_table(table_info)
    }

    pub fn metadata_untrack_table(&mut self, table_info: QualifiedTable) -> MetadataResult {
        self.metadata.untrack_table(table_info)
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn get_tracked_tables(&self) -> Vec<QualifiedTable> {
        self.metadata.get_tracked_tables()
    }
}
