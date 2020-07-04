use crate::ParseError;

#[derive(Debug, Clone)]
pub enum TagKind
{
    Location,
    Person,
    Event,
    Label,
    List,
    Activity,
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
            _ => Err(ParseError::new(format!("Invalid TagType field 0x{:0x}", val))),
        }
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
        }.to_owned()
    }
}
