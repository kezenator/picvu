use std::collections::HashMap;

use googlephotos::auth::AccessToken;

use picvudb::data::ExternalReference;

use crate::bulk::progress::ProgressSender;
use crate::bulk::sync::SyncError;

#[derive(Debug)]
struct MediaItem
{
    id: String,
    description: Option<String>,
    filename: String,
    creation_time: picvudb::data::Date,
    width: i64,
    height: i64,
}

#[derive(Debug)]
pub struct MediaItemDatabase
{
    id_to_media_item: HashMap<String, MediaItem>,
    filename_to_id_vec: HashMap<String, Vec<String>>,
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

            let response = googlephotos::api::raw::media_items_list(access_token, next_page_token)?;

            next_page_token = response.next_page_token;

            if let Some(media_items) = response.media_items
            {
                for media_item in media_items
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
            }

            if next_page_token.is_none()
            {
                break;
            }
        }

        sender.set(0.0, vec![format!("Loaded {} media items", id_to_media_item.len())]);

        Ok(MediaItemDatabase{ id_to_media_item, filename_to_id_vec })
    }

    pub fn filenames_with_multiple_media_items(&self) -> impl Iterator<Item = (&String, usize)>
    {
        self.filename_to_id_vec.iter()
            .filter(|e| e.1.len() > 1)
            .map(|e| (e.0, e.1.len()))
    }

    pub fn find_best_match(&self, filename: &String, date: &Option<picvudb::data::Date>) -> Option<ExternalReference>
    {
        if let Some(list) = self.filename_to_id_vec.get(filename)
        {
            let mut best_id = list[0].clone();

            if let Some(date) = date
            {
                let best_date = &self.id_to_media_item.get(&best_id).unwrap().creation_time;
                let mut best_diff = (best_date.to_chrono_utc() - date.to_chrono_utc()).num_milliseconds().abs();

                for i in 1..list.len()
                {
                    let next_date = self.id_to_media_item.get(&list[i]).unwrap().creation_time.clone();

                    let next_diff = (next_date.to_chrono_utc() - date.to_chrono_utc()).num_milliseconds().abs();

                    if next_diff < best_diff
                    {
                        best_id = list[i].clone();
                        best_diff = next_diff;
                    }
                }
            }

            return Some(ExternalReference::GooglePhotos{ id: best_id });
        }
        
        return None;
    }
}
