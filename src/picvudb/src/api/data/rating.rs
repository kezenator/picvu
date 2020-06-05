use crate::Error;

#[derive(Clone)]
pub enum Rating
{
    OneStar,
    TwoStars,
    ThreeStars,
    FourStars,
    FiveStars,
}

impl Rating
{
    pub fn from_num_stars(num: u8) -> Option<Self>
    {
        match num
        {
            1 => Some(Self::OneStar),
            2 => Some(Self::TwoStars),
            3 => Some(Self::ThreeStars),
            4 => Some(Self::FourStars),
            5 => Some(Self::FiveStars),
            _ => None,
        }
    }

    pub fn num_stars(&self) -> u8
    {
        match self
        {
            Self::OneStar => 1,
            Self::TwoStars => 2,
            Self::ThreeStars => 3,
            Self::FourStars => 4,
            Self::FiveStars => 5,
        }
    }

    pub(crate) fn to_db_field(&self) -> i32
    {
        self.num_stars() as i32
    }

    pub(crate) fn from_db_field(num: i32) -> Result<Self, Error>
    {
        Self::from_num_stars(num as u8)
            .ok_or(Error::DatabaseConsistencyError{ msg: format!("Invalid Rating: {} stars", num) })
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
        match self
        {
            Self::OneStar => "1 star",
            Self::TwoStars => "2 stars",
            Self::ThreeStars => "3 stars",
            Self::FourStars => "4 stars",
            Self::FiveStars => "5 stars",
        }.to_owned()
    }
}