use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum LocationSource
{
    UserProvided,
    CameraGps,
    ThirdPartyMetadata,
}

impl LocationSource
{
    pub(crate) fn to_db_field(&self) -> i32
    {
        match self
        {
            Self::UserProvided => 0x01,
            Self::CameraGps => 0x02,
            Self::ThirdPartyMetadata => 0x04,
        }
    }

    pub(crate) fn from_db_field(val: i32) -> Result<Self, ParseError>
    {
        match val
        {
            0x01 => Ok(Self::UserProvided),
            0x02 => Ok(Self::CameraGps),
            0x04 => Ok(Self::ThirdPartyMetadata),
            _ => Err(ParseError::new(format!("Invalid LocationSource field 0x{:0x}", val))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Location
{
    pub source: LocationSource,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

impl Location
{
    pub fn new(source: LocationSource, latitude: f64, longitude: f64, altitude: Option<f64>) -> Self
    {
        Location { source, latitude, longitude, altitude }
    }
}

impl ToString for Location
{
    fn to_string(&self) -> String
    {
        let mut result = format!("{},{}", self.latitude, self.longitude);

        if let Some(altitude) = self.altitude
        {
            result.push_str(&format!(",{:.0}m", altitude));
        }

        result
    }
}

impl FromStr for Location
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let parts = s.split(',').collect::<Vec<_>>();
        if parts.len() == 2 || parts.len() == 3
        {
            let latitude: f64 = parts[0].trim().parse().map_err(|_| ParseError::new("Invalid location data"))?;
            let longitude: f64 = parts[1].trim().parse().map_err(|_| ParseError::new("Invalid location data"))?;
            let mut altitude = None;

            if parts.len() == 3
            {
                let mut astr = parts[2].trim();
                if astr.ends_with('m')
                {
                    astr = &astr[..(astr.len() - 1)];
                }

                altitude = Some(astr.trim().parse().map_err(|_| ParseError::new("Invalid location data"))?);
            }

            return Ok(Location::new(LocationSource::UserProvided, latitude, longitude, altitude));
        }

        Err(ParseError::new("Invalid location data"))
    }
}

#[cfg(test)]
mod tests
{
    use super::Location;
    use super::LocationSource;

    #[test]
    fn test_location()
    {
        assert_eq!("1.234,-0.234,1234m".to_owned(), Location::new(LocationSource::UserProvided, 1.234, -0.234, Some(1234.0)).to_string());
        assert_eq!("1.234,-0.234,1234m".parse::<Location>(), Ok(Location::new(LocationSource::UserProvided, 1.234, -0.234, Some(1234.0))));
    }
}