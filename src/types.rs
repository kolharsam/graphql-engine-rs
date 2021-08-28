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
    pub Option<String>, // this is for any alias
    pub String,         // this is the actual field name
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

// Returns the string values of the `Field` names within a selection set
pub fn field_names_to_name_list(nl: &[FieldName]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for f_name in nl {
        names.push(f_name.1.clone());
    }

    names
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

// NOTE: GQLArgs is a simplified version of the
// AST's representation of arguments. Using
// such a structure only because it's easier
// to do operations. The AST is much more
// sophisticated and hence can become cumbersome
#[derive(Serialize, Clone, Debug)]
pub enum GQLArgType {
    // NOTE: supported for [distinct_on]
    String(String),
    // NOTE: supported for [limit, offset]
    Int(i64),
    // Float(f32)
}

type FieldArguments = indexmap::IndexMap<String, GQLArgType>;

impl GQLArgType {
    // NOTE: functions like the one's defined below
    // should be limited in usage since we can go
    // wrong quite easily since we're relying on the
    // developer's conscience over the type system
    // hence opening up an avenue for bugs and issues

    pub fn get_num(&self) -> i64 {
        if let GQLArgType::Int(num) = self {
            return *num;
        }
        // FIXME?: This should/would never happen
        // unless we use it incorrectly
        0
    }

    pub fn get_string(&self) -> String {
        if let GQLArgType::String(txt) = &self {
            return txt.to_string();
        }
        // FIXME?: This should/would never happen
        // unless we use it incorrectly
        "".to_string()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct FieldInfo {
    pub fields: Vec<FieldName>,
    pub root_field_arguments: indexmap::IndexMap<String, GQLArgType>,
}

impl FieldInfo {
    pub fn new(fields: Vec<FieldName>, args: FieldArguments) -> FieldInfo {
        FieldInfo {
            fields,
            root_field_arguments: args,
        }
    }
}

pub fn to_string_arg<'a>(
    arg_name: String,
    arg_val: &graphql_parser::query::Value<'a, &'a str>,
) -> Result<(String, GQLArgType), error::GQLRSError> {
    if let graphql_parser::query::Value::Enum(st) = arg_val {
        return Ok((arg_name, GQLArgType::String(st.to_string())));
    }

    Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
        "failed to parse string".to_string(),
    )))
}

pub fn to_int_arg<'a>(
    arg_name: String,
    arg_val: &graphql_parser::query::Value<'a, &'a str>,
) -> Result<(String, GQLArgType), error::GQLRSError> {
    if let graphql_parser::query::Value::Int(num) = arg_val {
        let inum = num.as_i64();
        match inum {
            Some(n) => return Ok((arg_name, GQLArgType::Int(n))),
            None => {
                return Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                    "int is overflown".to_string(),
                )))
            }
        }
    }

    Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
        "failed to parse int".to_string(),
    )))
}

// NOTE: these arguments are case sensitive, in case they're not
// these exactly we have every right to reject the query!
pub const SUPPORTED_STRING_GQL_ARGUMENTS: [&str; 1] = ["distinct_on"];
pub const SUPPORTED_INT_GQL_ARGUMENTS: [&str; 2] = ["offset", "limit"];
