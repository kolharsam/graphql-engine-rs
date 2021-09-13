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

// CAUTION: this method is unsafe!
pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

// To fetch the number of occurrences of an element in a list
pub fn get_frequency<T>(list: &[T], elem: &T) -> i32
where
    T: PartialEq + Eq,
{
    let mut count = 0;
    list.iter().for_each(|elm| {
        if elm == elem {
            count += 1;
        }
    });
    count
}
