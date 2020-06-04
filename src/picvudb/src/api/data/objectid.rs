use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectId(String);

impl ObjectId
{
    pub fn new(id: String) -> Self
    {
        ObjectId(id)
    }

    pub(crate) fn to_db_field(&self) -> &String
    {
        &self.0
    }
}

impl ToString for ObjectId
{
    fn to_string(&self) -> String
    {
        self.0.clone()
    }
}
