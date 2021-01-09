use std::cmp::{Ord, Ordering};

pub fn normalize(a: &str) -> String
{
    a.trim().to_lowercase()
}

pub fn cmp(a: &str, b: &str) -> Ordering
{
    let a = normalize(a);
    let b = normalize(b);

    a.cmp(&b)
}
