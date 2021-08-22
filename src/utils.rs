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

pub fn optional_fmap<T, U>(op: Option<T>, map_fn: fn(&T) -> U) -> Option<U> {
    if let Some(t) = op {
        return Some(map_fn(&t));
    }
    None
}
