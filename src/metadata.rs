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

type Tables = Vec<QualifiedTable>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Metadata {
    #[serde(rename = "source")]
    pub source_name: String,
    #[serde(default = "default_tables")]
    pub tables: Tables,
}

#[inline(always)]
fn default_tables() -> Tables {
    Vec::new()
}

pub type MetadataResult = Result<(), GQLRSError>;

impl Metadata {
    pub fn new(source_name: &str) -> Metadata {
        Metadata {
            source_name: String::from(source_name),
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

    pub fn track_table(&mut self, qualified_table: QualifiedTable) -> MetadataResult {
        if self.is_table_tracked(&qualified_table) {
            return Err(GQLRSError::new(GQLRSErrorType::TableAlreadyTracked(
                qualified_table.to_string(),
            )));
        }

        self.tables.push(QualifiedTable {
            schema_name: qualified_table.schema_name,
            table_name: qualified_table.table_name,
        });

        Ok(())
    }

    pub fn untrack_table(&mut self, qualified_table: QualifiedTable) -> MetadataResult {
        if !self.is_table_tracked(&qualified_table) {
            return Err(GQLRSError::new(GQLRSErrorType::TableNotFoundInMetadata(
                qualified_table.to_string(),
            )));
        }

        self.tables.retain(|table| {
            table.schema_name != qualified_table.schema_name
                && table.table_name != qualified_table.table_name
        });

        Ok(())
    }

    pub fn check_for_table_in_metadata(&self, table_name: &str) -> bool {
        false
    }
}

pub fn load_metadata(source_name: &str) -> Metadata {
    Metadata::new(source_name)
}
