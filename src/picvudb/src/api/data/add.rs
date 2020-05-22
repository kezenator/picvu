use crate::api::data::Date;

#[derive(Debug)]
pub struct Attachment
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum AdditionalData
{
    Photo
    {
        attachment: Attachment,
    },
    Video
    {
        attachment: Attachment,
    },
}

#[derive(Debug)]
pub struct ObjectData
{
    pub title: Option<String>,
    pub additional: AdditionalData,
}
