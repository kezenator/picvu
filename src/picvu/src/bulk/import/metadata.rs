use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Metadata
{
    title: String,
    description: String,
    image_views: String,
    creation_time: Timestamp,
    modification_time: Timestamp,
    geo_data: Option<GeoData>,
    geo_data_exif: Option<GeoData>,
    photo_taken_time: Timestamp,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Timestamp
{
    timestamp: String,
    formatted: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct GeoData
{
    latitude: f64,
    longitude: f64,
    altitude: f64,
    latitude_span: f64,
    longitude_span: f64,
}
