#[derive(Debug, Clone)]
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