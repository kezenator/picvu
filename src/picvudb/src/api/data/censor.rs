use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Censor
{
    FamilyFriendly,
    TastefulNudes,
    FullNudes,
    Explicit,
}

impl Censor
{
    pub(crate) fn to_db_field(&self) -> i32
    {
        match self
        {
            Self::FamilyFriendly => 0,
            Self::TastefulNudes => 1,
            Self::FullNudes => 2,
            Self::Explicit => 3,
        }
    }

    pub(crate) fn from_db_field(value: i32) -> Result<Self, Error>
    {
        match value
        {
            0 => Ok(Self::FamilyFriendly),
            1 => Ok(Self::TastefulNudes),
            2 => Ok(Self::FullNudes),
            3 => Ok(Self::Explicit),
            _ => Err(Error::DatabaseConsistencyError{ msg: format!("Invalid Censor value {}", value) }),
        }
    }
}

impl ToString for Censor
{
    fn to_string(&self) -> String
    {
        match self
        {
            Self::FamilyFriendly => "Family Friendly",
            Self::TastefulNudes => "Tasteful Nudes",
            Self::FullNudes => "Full Nudes",
            Self::Explicit => "Explicit",
        }.to_owned()
    }
}

impl std::str::FromStr for Censor
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s
        {
            "Family Friendly" => Ok(Self::FamilyFriendly),
            "Tasteful Nudes" => Ok(Self::TastefulNudes),
            "Full Nudes" => Ok(Self::FullNudes),
            "Explicit" => Ok(Self::Explicit),
            _ => Err(Error::DatabaseConsistencyError{msg : "Invalid Censor".to_owned() }),
        }
    }
}