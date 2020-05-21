use serde::Deserialize;

#[derive(Deserialize)]
pub struct Attachment
{
    pub hash: String,
}

#[derive(Deserialize)]
pub struct Thumbnail
{
    pub hash: String,
    pub size: u32,
}

#[derive(Deserialize)]
pub struct BulkImport
{
    pub folder: String,
}
