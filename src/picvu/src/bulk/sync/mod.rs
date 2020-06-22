use std::future::Future;
use std::pin::Pin;
use std::collections::{HashMap, HashSet};
use actix_web::web;

use googlephotos::auth::AccessToken;
use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;

pub mod albums;
pub mod error;
pub mod mediaitems;

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
                "Loading Google Photos albums".to_owned(),
                "Loading PicVu item data".to_owned(),
                "Creating new albums".to_owned(),
                "Updating albums".to_owned(),
            ];

            web::block(move ||
            {
                // Load albums
                let mut album_db =
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());

                    albums::AlbumDatabase::load_all(&access_token, &sender)?
                };

                println!("Album DB:\n:{:#?}", album_db);
                
                // Load objects
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());
                }

                let store = picvudb::Store::new(&db_uri)?;

                let objects =
                {
                    let stats = store.write_transaction(|ops|
                        {
                            picvudb::msgs::GetStatisticsRequest{}.execute(ops)
                        })?;

                    let msg = picvudb::msgs::GetObjectsRequest
                    {
                        query: picvudb::data::get::GetObjectsQuery::ByActivityDesc,
                        pagination: picvudb::data::get::PaginationRequest{ offset: 0, page_size: stats.num_objects },
                    };

                    let results = store.write_transaction(|ops|
                    {
                        msg.execute(ops)
                    })?;

                    assert_eq!(results.pagination_response.total, results.objects.len() as u64);

                    results.objects
                };

                // Creating albums
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());

                    let mut album_names = HashSet::new();

                    for object in objects.iter()
                    {
                        if let Some(picvudb::data::ExternalReference::GooglePhotos{..}) = &object.ext_ref
                        {
                            for tag in object.tags.iter()
                            {
                                album_names.insert(tag.name.clone());
                            }
                        }
                    }

                    album_db.create_albums(album_names, &access_token, &sender)?;
                }

                // Now put items into the albums
                {
                    let stage = stages[0].clone();
                    stages.remove(0);
                    sender.start_stage(stage, stages.clone());

                    sender.set(0.0, vec!["Analysing changes required...".to_owned()]);

                    let mut memberships = HashMap::new();

                    for object in objects.iter()
                    {
                        if let Some(picvudb::data::ExternalReference::GooglePhotos{id}) = &object.ext_ref
                        {
                            for tag in object.tags.iter()
                            {
                                memberships
                                    .entry(tag.name.clone())
                                    .or_insert(Vec::new())
                                    .push(id.clone());
                            }
                        }
                    }

                    album_db.apply_changes(memberships, &access_token, &sender)?;
                }

                Ok(())
                
            }).await?;

            Ok(())
        })
    }
}
