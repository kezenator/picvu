use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Metadata
{
    pub title: String,
    pub description: String,
    pub image_views: String,
    pub creation_time: Timestamp,
    pub modification_time: Option<Timestamp>,
    pub geo_data: Option<GeoData>,
    pub geo_data_exif: Option<GeoData>,
    pub photo_taken_time: Timestamp,
    pub photo_last_modified_time: Option<Timestamp>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Timestamp
{
    pub timestamp: String,
    pub formatted: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct GeoData
{
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub latitude_span: f64,
    pub longitude_span: f64,
}

pub fn parse_google_photos_takeout_metadata(json_bytes: Vec<u8>, err_path: &String) -> Result<Metadata, std::io::Error>
{
    let json_string = String::from_utf8(json_bytes)
        .map_err(|_| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Google Photos takeout metadata {} is not valid UTF-8", err_path)) })?;

    let metadata = serde_json::from_str::<Metadata>(&json_string)
        .map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Google Photos takeout metadata {} could not be decoded: {:?}", err_path, e)) })?;

    Ok(metadata)
}