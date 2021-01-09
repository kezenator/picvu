use std::cmp::{Ord, Ordering};

pub fn normalized_cmp(a: &str, b: &str) -> Ordering
{
    let a = a.trim().to_lowercase();
    let b = b.trim().to_lowercase();

    a.cmp(&b)
}
