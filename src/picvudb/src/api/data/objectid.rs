use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectId(pub(crate) String);

impl ObjectId
{
    pub fn new(id: String) -> Self
    {
        ObjectId(id)
    }
}

impl ToString for ObjectId
{
    fn to_string(&self) -> String
    {
        self.0.clone()
    }
}
