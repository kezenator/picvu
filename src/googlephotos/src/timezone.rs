use curl::easy::Easy;
use serde::Deserialize;
use url::Url;

#[derive(Debug)]
pub struct Timezone
{
    pub dst_offset_seconds: i32,
    pub raw_offset_seconds: i32,
    pub time_zone_id: String,
    pub time_zone_name: String,
}

#[derive(Debug)]
pub struct TimezoneError(String);

pub fn query_timezone(api_key: &str, latitude: f64, longitude: f64, timestamp: &chrono::DateTime<chrono::Utc>) -> Result<Timezone, TimezoneError>
{
    let mut url : Url = "https://maps.googleapis.com/maps/api/timezone/json".parse().expect("Can't decode hard-coded URL");
    url.query_pairs_mut().append_pair("location", &format!("{},{}", latitude, longitude));
    url.query_pairs_mut().append_pair("timestamp", &timestamp.timestamp().to_string());
    url.query_pairs_mut().append_pair("key", api_key);

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }

    println!("Data: {:?}", String::from_utf8_lossy(&data));

    let body = serde_json::from_slice::<JsonResponse>(&data)?;

    if body.status != "OK"
    {
        return Err(TimezoneError::new(format!("Bad response status: {:?}, msg={:?}", body.status, body.error_message.unwrap_or_default())));
    }

    let dst_offset_seconds = body.dst_offset.ok_or(TimezoneError::new("Bad response: missing DST offset".to_owned()))?;
    let raw_offset_seconds = body.raw_offset.ok_or(TimezoneError::new("Bad response: missing raw offset".to_owned()))?;
    let time_zone_id = body.time_zone_id.ok_or(TimezoneError::new("Bad response: missing Time Zone ID".to_owned()))?;
    let time_zone_name = body.time_zone_name.ok_or(TimezoneError::new("Bad response: missing Time Zone name".to_owned()))?;

    Ok(Timezone
    {
        dst_offset_seconds,
        raw_offset_seconds,
        time_zone_id,
        time_zone_name,
    })
}

impl TimezoneError
{
    fn new(s: String) -> Self
    {
        TimezoneError(s)
    }
}

impl From<curl::Error> for TimezoneError
{
    fn from(source: curl::Error) -> Self
    {
        TimezoneError::new(format!("HTTP error: {:?}", source))
    }
}

impl From<serde_json::Error> for TimezoneError
{
    fn from(source: serde_json::Error) -> Self
    {
        TimezoneError::new(format!("Response decode error: {:?}", source))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct JsonResponse
{
    dst_offset: Option<i32>,
    raw_offset: Option<i32>,
    status: String,
    error_message: Option<String>,
    time_zone_id: Option<String>,
    time_zone_name: Option<String>,
}
