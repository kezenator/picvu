use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Date
{
    timestamp: i64,
    timestring: String,
}

impl Date
{
    pub fn now() -> Self
    {
        let local = chrono::Local::now();
        let utc = local.with_timezone(&chrono::Utc);

        let timestamp = utc.timestamp_millis();
        let timestring = local.to_rfc3339();

        Date { timestamp, timestring }
    }

    pub fn from_chrono_datetime<T>(local: chrono::DateTime<T>) -> Self
        where T: chrono::offset::TimeZone,
            T::Offset: std::fmt::Display
    {
        let timestamp = local.with_timezone(&chrono::Utc).timestamp_millis();
        let timestring = local.to_rfc3339();

        Date { timestamp, timestring }
    }

    pub(crate) fn from_db_fields(timestamp: i64, timestring: String) -> Self
    {
        Date { timestamp, timestring }
    }

    pub(crate) fn from_db_timestamp(timestamp: i64) -> Self
    {
        let naive = chrono::naive::NaiveDateTime::from_timestamp(timestamp / 1000, ((timestamp % 1000) * 1000) as u32);
        let utc = chrono::DateTime::<chrono::Utc>::from_utc(naive, chrono::Utc);
        let timestring = utc.to_rfc3339();

        Date { timestamp, timestring }
    }

    pub fn to_db_timestamp(&self) -> i64
    {
        self.timestamp
    }

    pub fn to_db_timestring(&self) -> &String
    {
        &self.timestring
    }

    pub fn to_rfc3339(&self) -> String
    {
        self.timestring.clone()
    }

    pub fn to_chrono_fixed_offset(&self) -> chrono::DateTime<chrono::FixedOffset>
    {
        self.timestring.parse().unwrap()
    }
}
