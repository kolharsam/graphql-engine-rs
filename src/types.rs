use indexmap::IndexMap;
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

        format!(
            "{} AS {}",
            utils::dquote(self.1.as_str()),
            utils::dquote(&alias)
        )
    }

    pub fn name(&self) -> String {
        self.1.clone()
    }

    pub fn alias(&self) -> String {
        if let Some(alias) = self.0.clone() {
            return utils::dquote(&alias);
        }

        utils::dquote(&self.1)
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
    assert_eq!(
        new_field_name.to_sql(),
        "\"users\" AS \"users\"".to_string()
    );
}

#[test]
fn field_name_to_sql_with_alias() {
    let new_field_name = FieldName::new("users", Some("new_users".to_string()));
    assert_eq!(
        new_field_name.to_sql(),
        "\"users\" AS \"new_users\"".to_string()
    );
}

// NOTE: GQLArgs is a simplified version of the
// AST's representation of arguments. Using
// such a structure only because it's easier
// to do operations. The AST is much more
// sophisticated and hence can become cumbersome
#[derive(Serialize, Clone, Debug)]
// NOTE: We can keep increasing the types that this
// struct uses to ensure that we are writing type safe code.
// Certainly, there's no element of elegance to it. But,
// nevertheless should help the developer to write much simpler
// functions for the processing of the AST and SQL code
pub enum GQLArgType<T> {
    // NOTE: supported for [distinct_on]
    String(String),
    // NOTE: supported for [limit, offset]
    Int(i64),
    // NOTE: supported for [order_by]
    Object(IndexMap<String, T>),
}

pub type GQLArgTypeWithOrderBy = GQLArgType<OrderByOptions>;
type FieldArguments = indexmap::IndexMap<String, GQLArgTypeWithOrderBy>;

impl<T> GQLArgType<T> {
    // NOTE: functions like the one's defined below
    // should be limited in usage since we can go
    // wrong quite easily since we're relying on the
    // developer's conscience over the type system
    // hence opening up an avenue for bugs and issues

    // NOTE: for this function, it might be best to do
    // some `return-type polymorphism`

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

    pub fn get_object(&self) -> IndexMap<String, T>
    where
        T: Clone,
    {
        if let GQLArgType::Object(obj) = &self {
            return obj.clone();
        }
        // FIXME?: This should/would never happen
        // unless we use it incorrectly
        IndexMap::new()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct FieldInfo {
    pub fields: Vec<FieldName>,
    pub root_field_arguments: indexmap::IndexMap<String, GQLArgTypeWithOrderBy>,
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
) -> Result<(String, GQLArgTypeWithOrderBy), error::GQLRSError> {
    if let graphql_parser::query::Value::Enum(st) = arg_val {
        return Ok((arg_name, GQLArgType::String(st.to_string())));
    }

    Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
        format!("failed to parse argument {}", arg_name),
    )))
}

pub fn to_int_arg<'a>(
    arg_name: String,
    arg_val: &graphql_parser::query::Value<'a, &'a str>,
) -> Result<(String, GQLArgTypeWithOrderBy), error::GQLRSError> {
    if let graphql_parser::query::Value::Int(num) = arg_val {
        match num.as_i64() {
            Some(n) => return Ok((arg_name, GQLArgType::Int(n))),
            None => {
                return Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
                    "int is overflown".to_string(),
                )))
            }
        }
    }

    Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
        format!("failed to parse argument {}", arg_name),
    )))
}

// NOTE: This helper function will help us ensure that `order_by` argument is always
// supplied with legitimate keys. Which in this case would be the column names of the table
pub fn is_order_by_keys_valid<'a>(
    valid_keys: &[String],
    supplied_value: &graphql_parser::query::Value<'a, &'a str>,
) -> bool {
    if let graphql_parser::query::Value::Object(arg_bmap) = supplied_value {
        if arg_bmap.is_empty() {
            return false;
        }

        let keys_vec: Vec<&str> = arg_bmap.keys().cloned().collect();

        // Checking that keys are valid, in our case here would be
        // 1) to ensure that keys are all valid field names
        // 2) number of keys are at most = number of fields or columns on the table
        // 3) no duplicate keys found in the object
        return arg_bmap
            .keys()
            .all(|key| valid_keys.contains(&key.to_string()))
            && arg_bmap.len() <= valid_keys.len()
            && arg_bmap
                .keys()
                .all(|key| utils::get_frequency(&keys_vec, key) == 1);
    }

    false
}

pub fn to_object_arg<'a, T>(
    arg_name: String,
    arg_val: &graphql_parser::query::Value<'a, &'a str>,
    make_value_fn: fn(v: graphql_parser::query::Value<'a, &'a str>) -> Option<T>,
) -> Result<(String, GQLArgType<T>), error::GQLRSError> {
    if let graphql_parser::query::Value::Object(arg_bmap) = arg_val {
        let mut arg_map: IndexMap<String, T> = IndexMap::new();

        for (key_name, value) in arg_bmap.iter() {
            let arg_value = make_value_fn(value.clone());
            match arg_value {
                Some(v) => {
                    arg_map.insert(key_name.to_string(), v);
                }
                None => {
                    return Err(error::GQLRSError::new(error::GQLRSErrorType::InvalidInput(
                        format!(
                            "Incorrect value {} supplied to key {} in `{}` argument",
                            value, key_name, arg_name
                        ),
                    )));
                }
            }
        }

        return Ok((arg_name, GQLArgType::Object(arg_map)));
    }

    Err(error::GQLRSError::new(error::GQLRSErrorType::GenericError(
        format!("failed to parse argument {}", arg_name),
    )))
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OrderByOptions {
    Asc,
    AscNullsFirst,
    AscNullsLast,
    Desc,
    DescNullsFirst,
    DescNullsLast,
}

impl OrderByOptions {
    // NOTE: this method has to be implemented by all types that would
    //
    pub fn to_sql(&self) -> &str {
        match self {
            OrderByOptions::Asc => "ASC",
            OrderByOptions::AscNullsFirst => "ASC NULLS FIRST",
            OrderByOptions::AscNullsLast => "ASC NULLS LAST",
            OrderByOptions::Desc => "DESC",
            OrderByOptions::DescNullsFirst => "DESC NULLS FIRST",
            OrderByOptions::DescNullsLast => "DESC NULLS LAST",
        }
    }
}

pub fn from_parser_value_to_order_by_option<'a>(
    val: graphql_parser::query::Value<'a, &'a str>,
) -> Option<OrderByOptions> {
    if let graphql_parser::query::Value::Enum(str_val) = val {
        return to_order_by_option_value(str_val);
    }

    None
}

// TODO: use `serde` for this purpose instead
fn to_order_by_option_value(v: &str) -> Option<OrderByOptions> {
    match v {
        "asc" => Some(OrderByOptions::Asc),
        "asc_nulls_first" => Some(OrderByOptions::AscNullsFirst),
        "asc_nulls_last" => Some(OrderByOptions::AscNullsLast),
        "desc" => Some(OrderByOptions::Desc),
        "desc_nulls_first" => Some(OrderByOptions::DescNullsFirst),
        "desc_nulls_last" => Some(OrderByOptions::DescNullsLast),
        _ => None,
    }
}

// NOTE: these argument names are case sensitive, in case they're
// not these exactly we have every right to reject the query!
pub const SUPPORTED_INT_GQL_ARGUMENTS: [&str; 2] = ["offset", "limit"];
