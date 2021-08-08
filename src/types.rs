use serde::{Deserialize, Serialize};

#[path = "./error.rs"]
mod error;
#[path = "./utils.rs"]
mod utils;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QualifiedTable {
    #[serde(rename = "schema", default = "to_public_schema")]
    pub schema_name: String,
    #[serde(rename = "table")]
    pub table_name: String,
}

fn to_public_schema() -> String {
    "public".to_string()
}

impl std::fmt::Display for QualifiedTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}",
            utils::dqote(&self.schema_name.clone()),
            utils::dqote(&self.table_name.clone())
        )
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub source_name: String,
    pub connection_string: String,
    pub tables: Vec<QualifiedTable>,
}

impl Metadata {
    pub fn new(&self, source_name: String, connection_string: String) -> Metadata {
        Metadata {
            source_name,
            connection_string,
            tables: Vec::new(),
        }
    }

    pub fn update_connection_string(&mut self, connection_string: String) {
        // TODO: validate connection string?
        self.connection_string = connection_string;
    }

    fn is_table_tracked(&self, qualified_table: &QualifiedTable) -> bool {
        for table in self.tables.iter() {
            if table.schema_name == qualified_table.schema_name
                && table.table_name == qualified_table.table_name
            {
                return true;
            }
        }

        false
    }

    pub fn track_table(
        &mut self,
        qualified_table: &QualifiedTable,
    ) -> Result<(), error::GQLRSError> {
        if self.is_table_tracked(qualified_table) {
            return Err(error::GQLRSError {
                kind: error::GQLRSErrorType::TableAlreadyTracked(qualified_table.to_string()),
            });
        }
        self.tables.push(QualifiedTable {
            schema_name: qualified_table.schema_name.to_string(),
            table_name: qualified_table.table_name.to_string(),
        });
        Ok(())
    }

    pub fn untrack_table(
        &mut self,
        qualified_table: &QualifiedTable,
    ) -> Result<(), error::GQLRSError> {
        if !self.is_table_tracked(qualified_table) {
            return Err(error::GQLRSError {
                kind: error::GQLRSErrorType::TableNotFoundInMetadata(qualified_table.to_string()),
            });
        }
        // TODO: remove the table from the metadata
        Ok(())
    }
}
