use chrono::{DateTime, FixedOffset, NaiveDateTime, Local, Offset, Utc};

use crate::{Error, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Date
{
    Utc(DateTime<Utc>),
    FixedOffset(DateTime<FixedOffset>),
}

impl Date
{
    pub fn now() -> Self
    {
        let local = Local::now();
        let fixed = local.offset().fix();
        let value = local.with_timezone(&fixed);

        Date::FixedOffset(value)
    }

    pub fn from_rfc3339(s: &str) -> Result<Self, ParseError>
    {
        let fixed = chrono::DateTime::parse_from_rfc3339(s).map_err(|_| ParseError::new("Invalid Date/Time string"))?;

        Ok(Self::from_chrono_fixed(&fixed))
    }

    pub fn from_chrono_utc(utc: &chrono::DateTime<Utc>) -> Self
    {
        Date::Utc(utc.clone())
    }

    pub fn from_chrono_fixed<T>(local: &chrono::DateTime<T>) -> Self
        where T: chrono::offset::TimeZone,
            T::Offset: std::fmt::Display
    {
        let fixed = local.offset().fix();
        let value = local.with_timezone(&fixed);

        Date::FixedOffset(value)
    }

    pub(crate) fn from_db_fields(timestamp: i64, offset: Option<i32>) -> Result<Self, Error>
    {
        let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).ok_or(Error::DatabaseConsistencyError{msg: format!("Invalid Date/Time fields {} offset {:?}", timestamp, offset)})?;
        let utc = DateTime::<Utc>::from_utc(naive, Utc);

        match offset
        {
            Some(offset) =>
            {
                let fixed = FixedOffset::east_opt(offset).ok_or(Error::DatabaseConsistencyError{msg: format!("Invalid Date/Time fields {} offset {}", timestamp, offset)})?;
                let value = utc.with_timezone(&fixed);
        
                Ok(Date::FixedOffset(value))
            },
            None =>
            {
                Ok(Date::Utc(utc))
            }
        }
    }

    pub(crate) fn to_db_timestamp(&self) -> i64
    {
        match self
        {
            Date::Utc(utc) =>
            {
                utc.timestamp()
            },
            Date::FixedOffset(fixed) =>
            {
                let utc = fixed.with_timezone(&Utc);
                utc.timestamp()
            },
        }
    }

    pub(crate) fn to_db_offset(&self) -> Option<i32>
    {
        match self
        {
            Date::Utc(_) => None,
            Date::FixedOffset(fixed) => Some(fixed.offset().local_minus_utc())
        }
    }

    pub fn to_rfc3339(&self) -> String
    {
        match self
        {
            Date::Utc(utc) =>
            {
                utc.to_rfc3339()
            },
            Date::FixedOffset(fixed) =>
            {
                fixed.to_rfc3339()
            },
        }
    }

    pub fn to_chrono_fixed_offset(&self) -> DateTime<FixedOffset>
    {
        match self
        {
            Date::Utc(utc) =>
            {
                let fixed = utc.offset().fix();
                utc.with_timezone(&fixed)
            },
            Date::FixedOffset(fixed) =>
            {
                fixed.clone()
            },
        }
    }

    pub fn to_chrono_utc(&self) -> DateTime<Utc>
    {
        match self
        {
            Date::Utc(utc) =>
            {
                utc.clone()
            },
            Date::FixedOffset(fixed) =>
            {
                fixed.with_timezone(&Utc)
            },
        }
    }
}
