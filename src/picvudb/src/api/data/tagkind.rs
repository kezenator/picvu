use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TagKind
{
    Location,
    Person,
    Event,
    Label,
    List,
    Activity,
    Trash,
    Unsorted,
}

impl TagKind
{
    pub(crate) fn to_db_field(&self) -> i32
    {
        match self
        {
            Self::Location => 0x01,
            Self::Person => 0x02,
            Self::Event => 0x04,
            Self::Label => 0x08,
            Self::List => 0x10,
            Self::Activity => 0x20,
            Self::Trash => 0x40,
            Self::Unsorted => 0x80,
        }
    }

    pub(crate) fn from_db_field(val: i32) -> Result<Self, ParseError>
    {
        match val
        {
            0x01 => Ok(Self::Location),
            0x02 => Ok(Self::Person),
            0x04 => Ok(Self::Event),
            0x08 => Ok(Self::Label),
            0x10 => Ok(Self::List),
            0x20 => Ok(Self::Activity),
            0x40 => Ok(Self::Trash),
            0x80 => Ok(Self::Unsorted),
            _ => Err(ParseError::new(format!("Invalid TagType field 0x{:0x}", val))),
        }
    }

    pub fn values() -> Vec<Self>
    {
        vec![
            Self::Location,
            Self::Person,
            Self::Event,
            Self::Label,
            Self::List,
            Self::Activity,
            Self::Trash,
            Self::Unsorted,
        ]
    }

    pub fn is_system_kind(&self) -> bool
    {
        match self
        {
            Self::Location
                | Self::Person
                | Self::Event
                | Self::Label
                | Self::List
                | Self::Activity => false,
            Self::Trash
                | Self::Unsorted => true,
        }
    }

    pub fn system_name_unsorted() -> String
    {
        "Unsorted".to_owned()
    }

    pub fn system_name_trash() -> String
    {
        "Trash".to_owned()
    }
}

impl ToString for TagKind
{
    fn to_string(&self) -> String
    {
        match self
        {
            Self::Location => "Location",
            Self::Person => "Person",
            Self::Event => "Event",
            Self::Label => "Label",
            Self::List => "List",
            Self::Activity => "Activity",
            Self::Trash => "Trash",
            Self::Unsorted => "Unsorted",
        }.to_owned()
    }
}

impl FromStr for TagKind
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s
        {
            "Location" => Ok(TagKind::Location),
            "Person" => Ok(TagKind::Person),
            "Event" => Ok(TagKind::Event),
            "Label" => Ok(TagKind::Label),
            "List" => Ok(TagKind::List),
            "Activity" => Ok(TagKind::Activity),
            "Trash" => Ok(TagKind::Trash),
            "Unsorted" => Ok(TagKind::Unsorted),
            _ => Err(ParseError::new(format!("Invalid TagKind {:?}", s))),
        }
    }
}
