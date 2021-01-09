use serde::{Deserialize, Serialize};
use crate::ParseError;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Rating
{
    NotRated,
    OneStar,
    TwoStars,
    ThreeStars,
    FourStars,
    FiveStars,
}

impl Rating
{
    pub fn from_num_stars(num: i32) -> Result<Self, ParseError>
    {
        match num
        {
            0 => Ok(Self::NotRated),
            1 => Ok(Self::OneStar),
            2 => Ok(Self::TwoStars),
            3 => Ok(Self::ThreeStars),
            4 => Ok(Self::FourStars),
            5 => Ok(Self::FiveStars),
            _ => Err(ParseError::new(format!("Invalid rating number of stars: {}", num))),
        }
    }

    pub fn num_stars(&self) -> i32
    {
        match self
        {
            Self::NotRated => 0,
            Self::OneStar => 1,
            Self::TwoStars => 2,
            Self::ThreeStars => 3,
            Self::FourStars => 4,
            Self::FiveStars => 5,
        }
    }

    pub(crate) fn to_db_field(&self) -> Option<i32>
    {
        let num_stars = self.num_stars();

        match num_stars
        {
            0 => None,
            _ => Some(num_stars)
        }
    }

    pub(crate) fn from_db_field(num: Option<i32>) -> Result<Self, ParseError>
    {
        match num
        {
            Some(num) => Ok(Self::from_num_stars(num)?),
            None => Ok(Self::NotRated),
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
        match self
        {
            Self::NotRated => "Not Rated",
            Self::OneStar => "1 star",
            Self::TwoStars => "2 stars",
            Self::ThreeStars => "3 stars",
            Self::FourStars => "4 stars",
            Self::FiveStars => "5 stars",
        }.to_owned()
    }
}