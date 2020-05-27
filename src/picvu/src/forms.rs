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
pub struct MvImg
{
    pub hash: String,
    pub mp4_offset: usize,
}

#[derive(Deserialize)]
pub struct BulkImport
{
    pub folder: String,
}

#[derive(Deserialize)]
pub struct Pagination
{
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}
