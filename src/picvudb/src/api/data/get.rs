use crate::api::data::{Censor, Date, Dimensions, Duration, Location, ObjectId, Orientation, Rating};

#[derive(Debug, Clone)]
pub struct AttachmentMetadata
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub size: u64,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct ObjectMetadata
{
    pub id: ObjectId,
    pub created_time: Date,
    pub modified_time: Date,
    pub activity_time: Date,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<Rating>,
    pub censor: Censor,
    pub location: Option<Location>,
    pub attachment: AttachmentMetadata,
}

#[derive(Debug, Clone)]
pub struct PaginationRequest
{
    pub offset: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone)]
pub struct PaginationResponse
{
    pub offset: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub enum GetObjectsQuery
{
    ByActivityDesc,
    ByModifiedDesc,
    ByAttachmentSizeDesc,
    ByObjectId(ObjectId),
    NearLocationByActivityDesc{ location: Location, radius_meters: f64 },
    TitleNotesSearchByActivityDesc{ search: String },
}
