use serde::{Deserialize, Serialize};

use picvudb::data::{Censor, Date, Dimensions, Duration, Location, Orientation, Rating, TagKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: String,
    pub size: u64,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadata
{
    pub name: String,
    pub kind: TagKind,
    pub rating: Option<Rating>,
    pub censor: Censor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata
{
    pub created_time: Date,
    pub modified_time: Date,
    pub activity_time: Date,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<Rating>,
    pub censor: Censor,
    pub location: Option<Location>,
    pub attachment: AttachmentMetadata,
    pub tags: Vec<TagMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata
{
    pub version: String,
    pub start_time: Date,
    pub end_time: Date,
}

pub fn parse_object_metadata(json_bytes: Vec<u8>, err_path: &String) -> Result<ObjectMetadata, std::io::Error>
{
    let json_string = String::from_utf8(json_bytes)
        .map_err(|_| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Picvudb export metadata {} is not valid UTF-8", err_path)) })?;

    let metadata = serde_json::from_str::<ObjectMetadata>(&json_string)
        .map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Picvudb export metadata {} could not be decoded: {:?}", err_path, e)) })?;

    Ok(metadata)
}