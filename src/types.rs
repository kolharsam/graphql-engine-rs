use serde::{Deserialize, Serialize};

use crate::error;
use crate::utils;

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
            utils::dquote(&self.schema_name.clone()),
            utils::dquote(&self.table_name.clone())
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

// FieldName is a type exclusively for GraphQL
#[derive(Debug, Serialize, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct FieldName(
    Option<String>, // this is for any alias
    String,         // this is the actual field name
);

impl FieldName {
    pub fn new(field_name: &str, field_alias: Option<String>) -> FieldName {
        FieldName(field_alias, field_name.to_string())
    }

    pub fn to_sql(&self) -> String {
        let mut alias = String::from(&self.1);
        if let Some(a) = self.0.clone() {
            alias = a;
        }

        format!("{} AS {}", utils::dquote(self.1.as_str()), alias)
    }

    pub fn name(&self) -> String {
        self.1.clone()
    }

    pub fn alias(&self) -> String {
        if let Some(alias) = self.0.clone() {
            return alias;
        }

        self.1.clone()
    }
}

#[test]
fn field_name_to_sql_with_no_alias() {
    let new_field_name = FieldName::new("users", None);
    assert_eq!(new_field_name.to_sql(), "\"users\" AS users".to_string());
}

#[test]
fn field_name_to_sql_with_alias() {
    let new_field_name = FieldName::new("users", Some("new_users".to_string()));
    assert_eq!(
        new_field_name.to_sql(),
        "\"users\" AS new_users".to_string()
    );
}
