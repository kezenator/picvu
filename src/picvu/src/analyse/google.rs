use picvudb::data::{Date, Location};
use crate::analyse::tz::ExplicitTimezone;

pub use googlephotos::geocode::ReverseGeocode;

pub struct TimezoneInfo
{
    pub timezone: ExplicitTimezone,
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct GoogleCache
{
    api_key: String,
}

impl GoogleCache
{
    pub fn new<S: Into<String>>(api_key: S) -> Self
    {
        GoogleCache { api_key: api_key.into() }
    }

    pub fn get_timezone_for(&self, location: &Location, timestamp: &Date) -> Result<TimezoneInfo, String>
    {
        let result = googlephotos::timezone::query_timezone(
            &self.api_key,
            location.latitude,
            location.longitude,
            &timestamp.to_chrono_utc());

        let tz_info = result.map_err(|e| e.0)?;

        let offset = tz_info.dst_offset_seconds + tz_info.raw_offset_seconds;

        let fixed_offset = chrono::FixedOffset::east_opt(offset)
            .ok_or(format!("Google timezone returned invalid timezone offset {} + DST {}", tz_info.raw_offset_seconds, tz_info.dst_offset_seconds))?;

        let timezone = ExplicitTimezone::new(fixed_offset);
        let id = tz_info.time_zone_id;
        let name = tz_info.time_zone_name;

        Ok(TimezoneInfo{ timezone, id, name })
    }

    pub fn reverse_geocode(&self, location: &Location) -> Result<ReverseGeocode, String>
    {
        googlephotos::geocode::reverse_geocode(
            &self.api_key,
            location.latitude,
            location.longitude).map_err(|e| e.0)
    }
}
