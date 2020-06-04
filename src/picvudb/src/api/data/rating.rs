use crate::Error;

#[derive(Clone)]
pub struct Rating(u8);

impl Rating
{
    pub fn from_num_stars(num: u8) -> Option<Self>
    {
        match num
        {
            1 | 2 | 3 | 4 | 5 => Some(Self(num)),
            _ => None,
        }
    }

    pub fn num_stars(&self) -> u8
    {
        self.0
    }

    pub(crate) fn to_db_field(&self) -> i32
    {
        self.0 as i32
    }

    pub(crate) fn from_db_field(num: i32) -> Result<Self, Error>
    {
        match num
        {
            1 => Ok(Self(1)),
            2 => Ok(Self(2)),
            3 => Ok(Self(3)),
            4 => Ok(Self(4)),
            5 => Ok(Self(5)),
            _ => Err(Error::DatabaseConsistencyError{ msg: format!("Invalid Rating: {} stars", num) }),
        }
    }
}

impl std::fmt::Debug for Rating
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{}", self.to_string())
    }
}

impl ToString for Rating
{
    fn to_string(&self) -> String
    {
        match self.0
        {
            1 => "1 star",
            2 => "2 stars",
            3 => "3 stars",
            4 => "4 stars",
            5 => "5 stars",
            _ => panic!(),
        }.to_owned()
    }
}