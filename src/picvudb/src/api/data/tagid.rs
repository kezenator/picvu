use serde::Serialize;
use std::str::FromStr;

use crate::data::id;
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TagId(i64);

impl TagId
{
    pub fn try_new(id: String) -> Result<Self, ParseError>
    {
        id::decode(&id, "t").map(|val| TagId(val))
    }

    pub(crate) fn to_db_field(&self) -> i64
    {
        self.0
    }

    pub(crate) fn from_db_field(val: i64) -> TagId
    {
        TagId(val)
    }
}

impl ToString for TagId
{
    fn to_string(&self) -> String
    {
        id::encode(self.0, "t")
    }
}

impl FromStr for TagId
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Self::try_new(s.into())
    }
}
