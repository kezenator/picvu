use chrono::{DateTime, FixedOffset, NaiveDateTime, Local, Offset, Utc};

use crate::Error;

#[derive(Debug, Clone)]
pub struct Date(DateTime<FixedOffset>);

impl Date
{
    pub fn now() -> Self
    {
        let local = Local::now();
        let fixed = local.offset().fix();
        let value = local.with_timezone(&fixed);

        Date(value)
    }

    pub fn from_chrono<T>(local: &chrono::DateTime<T>) -> Self
        where T: chrono::offset::TimeZone,
            T::Offset: std::fmt::Display
    {
        let fixed = local.offset().fix();
        let value = local.with_timezone(&fixed);

        Date(value)
    }

    pub(crate) fn from_db_fields(timestamp: i64, offset: i32) -> Result<Self, Error>
    {
        let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).ok_or(Error::DatabaseConsistencyError{msg: format!("Invalid Date/Time fields {} offset {}", timestamp, offset)})?;
        let utc = DateTime::<Utc>::from_utc(naive, Utc);
        let fixed = FixedOffset::east_opt(offset).ok_or(Error::DatabaseConsistencyError{msg: format!("Invalid Date/Time fields {} offset {}", timestamp, offset)})?;
        let value = utc.with_timezone(&fixed);

        Ok(Date(value))
    }

    pub(crate) fn to_db_timestamp(&self) -> i64
    {
        let utc = self.0.with_timezone(&Utc);
        utc.timestamp()
    }

    pub(crate) fn to_db_offset(&self) -> i32
    {
        self.0.offset().local_minus_utc()
    }

    pub fn to_rfc3339(&self) -> String
    {
        self.0.to_rfc3339()
    }

    pub fn to_chrono_fixed_offset(&self) -> DateTime<FixedOffset>
    {
        self.0.clone()
    }

    pub fn to_chrono_utc(&self) -> DateTime<Utc>
    {
        self.0.with_timezone(&Utc)
    }
}
