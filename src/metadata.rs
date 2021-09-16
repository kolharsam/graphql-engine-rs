use serde::{Deserialize, Serialize};

use crate::error::{GQLRSError, GQLRSErrorType};
use crate::utils::dquote;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QualifiedTable {
    #[serde(rename = "schema", default = "public_schema")]
    pub schema_name: String,
    #[serde(rename = "table")]
    pub table_name: String,
}

#[inline(always)]
fn public_schema() -> String {
    String::from("public")
}

impl std::fmt::Display for QualifiedTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}",
            dquote(&self.schema_name),
            dquote(&self.table_name)
        )
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub source_name: String,
    pub tables: Vec<QualifiedTable>,
}

impl Metadata {
    pub fn new(&self, source_name: String) -> Metadata {
        Metadata {
            source_name,
            tables: Vec::new(),
        }
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

    pub fn track_table(&mut self, qualified_table: &QualifiedTable) -> Result<(), GQLRSError> {
        if self.is_table_tracked(qualified_table) {
            return Err(GQLRSError {
                kind: GQLRSErrorType::TableAlreadyTracked(qualified_table.to_string()),
            });
        }
        self.tables.push(QualifiedTable {
            schema_name: qualified_table.schema_name.to_string(),
            table_name: qualified_table.table_name.to_string(),
        });
        Ok(())
    }

    pub fn untrack_table(&mut self, qualified_table: &QualifiedTable) -> Result<(), GQLRSError> {
        if !self.is_table_tracked(qualified_table) {
            return Err(GQLRSError {
                kind: GQLRSErrorType::TableNotFoundInMetadata(qualified_table.to_string()),
            });
        }
        // TODO: remove the table from the metadata
        Ok(())
    }
}
