use crate::api::data::{Date, ObjectId, ObjectType};

#[derive(Debug)]
pub struct AttachmentMetadata
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug)]
pub struct PhotoMetadata
{
    pub attachment: AttachmentMetadata,
}

#[derive(Debug)]
pub struct VideoMetadata
{
    pub attachment: AttachmentMetadata,
}

#[derive(Debug)]
pub enum AdditionalMetadata
{
    Photo(PhotoMetadata),
    Video(VideoMetadata),
}

#[derive(Debug)]
pub struct ObjectMetadata
{
    pub id: ObjectId,
    pub added: Date,
    pub changed: Date,
    pub obj_type: ObjectType,
    pub title: Option<String>,
    pub additional: AdditionalMetadata,
}
