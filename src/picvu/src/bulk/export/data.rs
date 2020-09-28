use serde::Serialize;

use picvudb::data::{Censor, Date, Dimensions, Duration, Location, Orientation, Rating, TagKind};

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct TagMetadata
{
    pub name: String,
    pub kind: TagKind,
    pub rating: Option<Rating>,
    pub censor: Censor,
}

#[derive(Debug, Clone, Serialize)]
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
