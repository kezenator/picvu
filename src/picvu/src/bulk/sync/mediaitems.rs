use std::collections::HashMap;

use googlephotos::auth::AccessToken;

use picvudb::data::ExternalReference;

use crate::bulk::progress::ProgressSender;
use crate::bulk::sync::SyncError;

#[derive(Debug)]
pub struct MediaItem
{
    pub id: String,
    pub description: Option<String>,
    pub filename: String,
    pub creation_time: picvudb::data::Date,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug)]
pub struct MediaItemDatabase
{
    pub id_to_media_item: HashMap<String, MediaItem>,
    pub filename_to_id_vec: HashMap<String, Vec<String>>,
}

impl MediaItemDatabase
{
    pub fn empty() -> MediaItemDatabase
    {
        MediaItemDatabase
        {
            id_to_media_item: HashMap::new(),
            filename_to_id_vec: HashMap::new(),
        }
    }

    pub fn load_all(access_token: &AccessToken, sender: &ProgressSender) -> Result<MediaItemDatabase, SyncError>
    {
        let mut id_to_media_item = HashMap::new();
        let mut filename_to_id_vec = HashMap::new();

        let mut next_page_token = None;

        loop
        {
            sender.set(0.0, vec![format!("Loaded {} media items", id_to_media_item.len())]);

            let mut response = googlephotos::api::raw::media_items_list(access_token, next_page_token)?;

            next_page_token = response.next_page_token;

            for media_item in response.media_items.drain(..)
            {
                let filename = media_item.filename.clone();
                let filename_list = filename_to_id_vec.entry(filename).or_insert(Vec::new());
                filename_list.push(media_item.id.clone());

                let creation_time: chrono::DateTime<chrono::Utc> = media_item.media_metadata.creation_time.parse()
                    .map_err(|_| SyncError::new_parse_err(format!("Invalid Google Photos creation_time {:?}", media_item.media_metadata.creation_time)))?;
                let width: i64 = media_item.media_metadata.width.parse()
                    .map_err(|_| SyncError::new_parse_err(format!("Invalid Google Photos width {:?}", media_item.media_metadata.width)))?;
                let height: i64 = media_item.media_metadata.height.parse()
                    .map_err(|_| SyncError::new_parse_err(format!("Invalid Google Photos height {:?}", media_item.media_metadata.height)))?;

                let media_item = MediaItem
                {
                    id: media_item.id,
                    description: media_item.description,
                    filename: media_item.filename,
                    creation_time: picvudb::data::Date::from_chrono_utc(&creation_time),
                    width: width,
                    height: height,
                };

                id_to_media_item.insert(media_item.id.clone(), media_item);
            }

            if next_page_token.is_none()
            {
                break;
            }
        }

        sender.set(0.0, vec![format!("Loaded {} media items", id_to_media_item.len())]);

        Ok(MediaItemDatabase{ id_to_media_item, filename_to_id_vec })
    }

    pub fn find_best_match(&self, filename: &String, _date: &Option<picvudb::data::Date>) -> Option<ExternalReference>
    {
        if let Some(list) = self.filename_to_id_vec.get(filename)
        {
            return Some(ExternalReference::GooglePhotos{ id: list[0].clone() });
        }
        
        return None;
    }
}
