use std::future::Future;
use std::pin::Pin;
use actix_web::web;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;

pub struct DeleteTagBulkOp
{
    db_uri: String,
    tag_id: picvudb::data::TagId,
}

impl DeleteTagBulkOp
{
    pub fn new(db_uri: String, tag_id: picvudb::data::TagId) -> Self
    {
        DeleteTagBulkOp
        {
            db_uri,
            tag_id,
        }
    }
}

impl BulkOperation for DeleteTagBulkOp
{
    type Error = actix_rt::blocking::BlockingError<picvudb::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<(), Self::Error>>>>;

    fn name(&self) -> String
    {
        "Delete tag".to_owned()
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        let db_uri = self.db_uri;
        let tag_id = self.tag_id;

        Box::pin(async move
        {
            web::block(move ||
            {
                sender.start_stage("Delete tag".to_owned(), vec![]);

                let store = picvudb::Store::new(&db_uri)?;

                let objects =
                {
                    let query = picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag_id.clone() };

                    let msg = picvudb::msgs::GetObjectsRequest
                    {
                        query: query.clone(),
                        pagination: None,
                    };

                    let results = store.write_transaction(|ops|
                    {
                        msg.execute(ops)
                    })?;

                    assert_eq!(results.pagination_response.total, results.objects.len() as u64);

                    results.objects
                };

                // Delete this tag from each object

                let num_objects = objects.len();
                let mut done = 0;

                for object in objects
                {
                    done += 1;

                    sender.set(
                        (done as f64) / (num_objects as f64) * 100.0,
                        vec![format!("Deleted from {} of {} items", done, num_objects)]);

                    let msg = picvudb::msgs::UpdateObjectTagsRequest
                    {
                        object_id: object.id,
                        remove: vec![tag_id.clone()],
                        add: Vec::new(),
                    };

                    store.write_transaction(|ops|
                        {
                            msg.execute(ops)
                        })?;
                }

                sender.set(100.0, vec!["Completed".to_owned()]);

                Ok(())
                
            }).await?;

            Ok(())
        })
    }
}
