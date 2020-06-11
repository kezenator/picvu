use std::str::FromStr;
use crate::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub struct Location
{
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

impl Location
{
    pub fn new(latitude: f64, longitude: f64, altitude: Option<f64>) -> Self
    {
        Location { latitude, longitude, altitude }
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

            return Ok(Location::new(latitude, longitude, altitude));
        }

        Err(ParseError::new("Invalid location data"))
    }
}

#[cfg(test)]
mod tests
{
    use super::Location;

    #[test]
    fn test_location()
    {
        assert_eq!("1.234,-0.234,1234m".to_owned(), Location::new(1.234, -0.234, Some(1234.0)).to_string());
        assert_eq!("1.234,-0.234,1234m".parse::<Location>(), Ok(Location::new(1.234, -0.234, Some(1234.0))));
    }
}