use serde::Serialize;

use crate::data::id;
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectId(i64);

impl ObjectId
{
    pub fn try_new(id: String) -> Result<Self, ParseError>
    {
        id::decode(&id, "o").map(|val| ObjectId(val))
    }

    pub(crate) fn to_db_field(&self) -> i64
    {
        self.0
    }

    pub(crate) fn from_db_field(val: i64) -> ObjectId
    {
        ObjectId(val)
    }
}

impl ToString for ObjectId
{
    fn to_string(&self) -> String
    {
        id::encode(self.0, "o")
    }
}
