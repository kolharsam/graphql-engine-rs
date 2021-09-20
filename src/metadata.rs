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

type Tables = Option<Vec<QualifiedTable>>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub source_name: String,
    pub tables: Tables,
}

impl Metadata {
    pub fn new(source_name: String) -> Metadata {
        Metadata {
            source_name,
            tables: None,
        }
    }

    fn is_table_tracked(&self, qualified_table: &QualifiedTable) -> bool {
        if let Some(tables) = &self.tables {
            for table in tables.iter() {
                if table.schema_name == qualified_table.schema_name
                    && table.table_name == qualified_table.table_name
                {
                    return true;
                }
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

        if let Some(tables) = self.tables.as_mut() {
            tables.push(QualifiedTable {
                schema_name: qualified_table.schema_name.to_string(),
                table_name: qualified_table.table_name.to_string(),
            });
        }

        Ok(())
    }

    pub fn untrack_table(&mut self, qualified_table: &QualifiedTable) -> Result<(), GQLRSError> {
        if !self.is_table_tracked(qualified_table) {
            return Err(GQLRSError {
                kind: GQLRSErrorType::TableNotFoundInMetadata(qualified_table.to_string()),
            });
        }

        if let Some(tables) = self.tables.as_mut() {
            tables.retain(|table| {
                table.schema_name != qualified_table.schema_name
                    && table.table_name != qualified_table.table_name
            });
        }

        Ok(())
    }
}

pub fn load_metadata(source_name: String) -> Metadata {
    Metadata::new(source_name)
}
