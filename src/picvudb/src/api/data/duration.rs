use crate::Error;

#[derive(Clone)]
pub struct Duration(u32);

impl Duration
{
    pub fn from_seconds(seconds: u32) -> Self
    {
        Duration(seconds)
    }

    pub(crate) fn to_db_field(&self) -> i32
    {
        self.0 as i32
    }

    pub(crate) fn from_db_field(val: Option<i32>) -> Result<Option<Self>, Error>
    {
        match val
        {
            None => Ok(None),
            Some(val) =>
            {
                if val >= 0
                {
                    Ok(Some(Duration(val as u32)))
                }
                else
                {
                    Err(Error::DatabaseConsistencyError{ msg: format!("Invalid Duration: {}", val) })
                }
            },
        }
    }
}

impl std::fmt::Debug for Duration
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{}", self.to_string())
    }
}

impl ToString for Duration
{
    fn to_string(&self) -> String
    {
        let hours = self.0 / 3600;
        let mins = (self.0 / 60) % 60;
        let secs = self.0 % 60;

        format!("{}:{:02}:{:02}", hours, mins, secs)
    }
}
