use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use actix_web::web;

use googlephotos::auth::AccessToken;
use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;

pub mod error;
pub use error::SyncError;

pub struct GooglePhotosSync
{
    access_token: AccessToken,
    db_uri: String,
}

impl GooglePhotosSync
{
    pub fn new(access_token: AccessToken, db_uri: String) -> Self
    {
        GooglePhotosSync
        {
            access_token,
            db_uri,
        }
    }
}

impl BulkOperation for GooglePhotosSync
{
    type Error = actix_rt::blocking::BlockingError<error::SyncError>;
    type Future = Pin<Box<dyn Future<Output=Result<(), Self::Error>>>>;

    fn name(&self) -> String
    {
        "Google Photos Sync".to_owned()
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        let access_token = self.access_token;
        let db_uri = self.db_uri;

        Box::pin(async move
        {
            let mut stages = vec![
                "Loading Google Photos media items".to_owned(),
                "Loading Google Photos albums".to_owned(),
                "Loading Google Photos album contents".to_owned(),
                "Loading PicVu item data".to_owned(),
                "Updating items".to_owned(),
            ];

            web::block(move ||
            {
                // Load media items
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());
                }

                let mut gp_id_to_media_item = HashMap::new();
                let mut gp_filename_to_id_list = HashMap::new();

                {
                    let mut next_page_token = None;

                    loop
                    {
                        sender.set(0.0, vec![format!("Loaded {} media items", gp_id_to_media_item.len())]);

                        let mut response = googlephotos::api::raw::media_items_list(&access_token, next_page_token)?;

                        next_page_token = response.next_page_token;

                        for media_item in response.media_items.drain(..)
                        {
                            let filename = media_item.filename.clone();
                            let filename_list = gp_filename_to_id_list.entry(filename).or_insert(Vec::new());
                            filename_list.push(media_item.id.clone());

                            gp_id_to_media_item.insert(media_item.id.clone(), media_item);
                        }

                        if next_page_token.is_none()
                        {
                            break;
                        }
                        // TODO - remove
                        break;
                    }

                    sender.set(0.0, vec![format!("Loaded {} media items", gp_id_to_media_item.len())]);
                }

                // Load albums
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());
                }
                
                // Load album contents
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());
                }

                // Load objects
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());
                }

                let store = picvudb::Store::new(&db_uri)?;

                {
                    let stats = store.write_transaction(|ops|
                        {
                            picvudb::msgs::GetStatisticsRequest{}.execute(ops)
                        })?;

                    let msg = picvudb::msgs::GetObjectsRequest
                    {
                        query: picvudb::data::get::GetObjectsQuery::ByModifiedDesc,
                        pagination: picvudb::data::get::PaginationRequest{ offset: 0, page_size: stats.num_objects },
                    };

                    let mut results = store.write_transaction(|ops|
                    {
                        msg.execute(ops)
                    })?;

                    assert_eq!(results.pagination_response.total, results.objects.len() as u64);

                    let mut warnings = Vec::new();

                    for object in results.objects.drain(..)
                    {
                        sender.set(0.0, warnings.clone());

                        let filename = match object.additional
                        {
                            picvudb::data::get::AdditionalMetadata::Photo(photo) =>
                            {
                                photo.attachment.filename.clone()
                            },
                            picvudb::data::get::AdditionalMetadata::Video(video) =>
                            {
                                video.attachment.filename.clone()
                            },
                        };

                        //let mut id = None;

                        if let Some(id_list) = gp_filename_to_id_list.get(&filename)
                        {
                            for id in id_list.iter()
                            {
                                if let Some(gp_media_item) = gp_id_to_media_item.get(id)
                                {
                                    warnings.push(format!("{} obj {} activity {} id {} created {}",
                                        filename, object.id.to_string(), object.activity_time.to_rfc3339(), id, gp_media_item.media_metadata.creation_time));
                                }
                            }
                        }
                    }
                }

                Ok(())
                
            }).await?;

            Ok(())
        })
    }
}
