use convert_case::{Case, Casing};

pub fn dquote(s: &str) -> String {
    format!("\"{}\"", s)
}

pub fn to_snake_case(s: &str) -> String {
    s.to_case(Case::Snake)
}

pub fn to_camel_case(s: &str) -> String {
    s.to_case(Case::Camel)
}

pub fn to_upper_case(s: &str) -> String {
    s.to_case(Case::Upper)
}

pub fn to_lower_case(s: &str) -> String {
    s.to_case(Case::Lower)
}

// UNSAFE NOTICE: this option is very unsafe!! use with caution!
pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

// NOTE: This is a bit of a hack, but it's the only way I could get the order 
// of the keys to match that of the query
pub fn remap_json(current_json: serde_json::Value, key_order: &Vec<String>) -> serde_json::Value {
    if current_json.is_object() {
        let mut new_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

        for key in key_order.iter() {
            match current_json.get(key) {
                Some(v) => {
                    new_json.insert(key.to_string(), v.clone());
                },
                None => {},
            }
        }

        return serde_json::Value::Object(new_json);
    } else if current_json.is_array() {
        let mut new_json_arr: Vec<serde_json::Value> = Vec::new();

        // NOTE: watch out for the unwrap here
        let current_json_vec: Vec<serde_json::Value> = serde_json::from_value(current_json).unwrap();

        for current_obj in current_json_vec.iter() {
            let mut new_obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for key in key_order.iter() {
                match current_obj.get(key) {
                    Some(v) => {
                        new_obj.insert(key.to_string(), v.clone());
                    },
                    None => {},
                }
            }
            new_json_arr.push(serde_json::Value::Object(new_obj));
        }

        return serde_json::Value::Array(new_json_arr);
    }

    // NOTE: worst case scenario, ideally we shouldn't get here
    serde_json::Value::Null
}
