use convert_case::{Case, Casing};

pub fn dqote(s: &str) -> String {
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
