use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaItemsListResponse
{
    pub media_items: Option<Vec<MediaItem>>,
    pub next_page_token: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaItem
{
    pub id: String,
    pub description: Option<String>,
    pub product_url: String,
    pub base_url: String,
    pub mime_type: String,
    pub media_metadata: MediaMetadata,
    pub filename: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MediaMetadata
{
    pub creation_time: String,
    pub width: String,
    pub height: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SharedAlbumOptions
{
    pub is_collaborative: bool,
    pub is_commentable: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AlbumShareInfo
{
    pub shared_album_options: SharedAlbumOptions,
    pub shareable_url: String,
    pub share_token: String,
    pub is_joined: bool,
    pub is_owned: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Album
{
    pub id: String,
    pub title: String,
    pub product_url: String,
    pub is_writeable: Option<bool>,
    pub share_info: Option<AlbumShareInfo>,
    pub media_items_count: Option<String>,
    pub cover_photo_base_url: Option<String>,
    pub cover_photo_media_item_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AlbumsListResponse
{
    pub albums: Vec<Album>,
    pub next_page_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CreateAlbumInfo
{
    pub title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CreateAlbumRequest
{
    pub album: CreateAlbumInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SearchRequest
{
    pub album_id: Option<String>,
    pub page_size: u64,
    pub page_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AlbumModifyRequest
{
    pub media_item_ids: Vec<String>,
}
