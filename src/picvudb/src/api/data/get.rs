use crate::api::data::{Date, ObjectId, ObjectType};

#[derive(Debug, Clone)]
pub struct AttachmentMetadata
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct PhotoMetadata
{
    pub attachment: AttachmentMetadata,
}

#[derive(Debug, Clone)]
pub struct VideoMetadata
{
    pub attachment: AttachmentMetadata,
}

#[derive(Debug, Clone)]
pub enum AdditionalMetadata
{
    Photo(PhotoMetadata),
    Video(VideoMetadata),
}

#[derive(Debug, Clone)]
pub struct ObjectMetadata
{
    pub id: ObjectId,
    pub added: Date,
    pub changed: Date,
    pub obj_type: ObjectType,
    pub title: Option<String>,
    pub additional: AdditionalMetadata,
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
    ByModifiedDesc,
    ByAttachmentSizeDesc,
    ByObjectId(ObjectId),
}