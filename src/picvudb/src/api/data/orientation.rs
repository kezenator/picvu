use serde::Serialize;
use crate::Error;

#[derive(Debug, Clone, Serialize)]
pub enum Orientation
{
    Straight,
    RotatedRight,
    UpsideDown,
    RotatedLeft,
}

impl Orientation
{
    pub(crate) fn to_db_field(&self) -> i32
    {
        match self
        {
            Self::Straight => 1,
            Self::RotatedRight => 2,
            Self::UpsideDown => 3,
            Self::RotatedLeft => 4,
        }
    }

    pub(crate) fn from_db_field(value: Option<i32>) -> Result<Option<Self>, Error>
    {
        match value
        {
            None => Ok(None),
            Some(value) =>
            {
                match value
                {
                    1 => Ok(Some(Self::Straight)),
                    2 => Ok(Some(Self::RotatedRight)),
                    3 => Ok(Some(Self::UpsideDown)),
                    4 => Ok(Some(Self::RotatedLeft)),
                    _ => Err(Error::DatabaseConsistencyError{ msg: format!("Invalid Orientation: {}", value) }),
                }
            }
        }
    }
}

impl ToString for Orientation
{
    fn to_string(&self) -> String
    {
        match self
        {
            Self::Straight => "Straight",
            Self::RotatedRight => "Rotated Right",
            Self::UpsideDown => "Upside Down",
            Self::RotatedLeft => "Rotated Left",
        }.to_owned()
    }
}
