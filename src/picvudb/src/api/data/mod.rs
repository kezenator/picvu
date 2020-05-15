use serde::Serialize;

mod date;
pub use date::Date;

#[derive(Debug, Serialize)]
pub struct Object
{
    pub id: String,
    pub added: Date,
    pub changed: Date,
    pub label: String,
}
