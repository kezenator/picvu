use crate::Error;

#[derive(Debug, Clone)]
pub enum ObjectType
{
    Photo,
    Video,
}

impl ObjectType
{
    pub(crate) fn to_db_string(&self) -> String
    {
        match self
        {
            Self::Photo => "photo".to_owned(),
            Self::Video => "video".to_owned(),
        }
    }

    pub (crate) fn from_db_string(val: &str) -> Result<Self, Error>
    {
        match val
        {
            "photo" => Ok(Self::Photo),
            "video" => Ok(Self::Video),
            _ =>
            {
                Err(Error::DatabaseConsistencyError{
                    msg: format!("Unknown obj_type value \"{}\"", val),
                })
            }
        }
    }
}

impl std::fmt::Display for ObjectType
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{:?}", self)
    }
}