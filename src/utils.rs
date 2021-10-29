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

/// CAUTION: this method is unsafe!
pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// To fetch the number of occurrences of an element in a list
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

/// this method is based on the Haskell function `either`
/// `Either` is analogous to `Result` in Rust
/// This is what the Haskell function `either` does:
/// either :: (a -> c) -> (b -> c) -> Either a b -> c
pub fn map_result<A, B, T, F, G>(err_fn: F, ok_fn: G, result: Result<A, B>) -> T
where
    F: FnOnce(B) -> T,
    G: FnOnce(A) -> T,
{
    match result {
        Ok(ok) => ok_fn(ok),
        Err(err) => err_fn(err),
    }
}
