use std::future::Future;
use std::pin::Pin;
use actix_web::web;
use chrono::Datelike;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;
use crate::format;

mod data;
mod error;
mod writer;

use error::*;
use writer::ExportWriter;

pub struct Export
{
    folder_path: String,
    db_uri: String,
}

impl Export
{
    pub fn new(folder_path: String, db_uri: String) -> Self
    {
        Export
        {
            folder_path,
            db_uri,
        }
    }
}

impl BulkOperation for Export
{
    type Error = actix_rt::blocking::BlockingError<ExportError>;
    type Future = Pin<Box<dyn Future<Output=Result<(), Self::Error>>>>;

    fn name(&self) -> String
    {
        format!("Bulk Export: {}", self.folder_path)
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        Box::pin(async move
        {
            web::block(move || -> Result<(), ExportError>
            {
                sender.start_stage("Loading objects".to_owned(), vec!["Exporting".to_owned(), "Cleaning Up".to_owned()]);

                let mut writer = writer::FileExportWriter::new(self.folder_path)?;

                let store = picvudb::Store::new(&self.db_uri)?;

                let num_objects_req = picvudb::msgs::GetNumObjectsRequest
                {
                    query: picvudb::data::get::GetObjectsQuery::ByActivityDesc,
                };

                let num_objects_resp = store.write_transaction(|ops|
                    {
                        num_objects_req.execute(ops)
                    })?;

                let get_objects_req = picvudb::msgs::GetObjectsRequest
                {
                    query: picvudb::data::get::GetObjectsQuery::ByActivityDesc,
                    pagination: Some(picvudb::data::get::PaginationRequest
                    {
                        offset: 0,
                        page_size: num_objects_resp.num_objects,
                    }),
                };

                let get_objects_resp = store.write_transaction(|ops|
                    {
                        get_objects_req.execute(ops)
                    })?;

                let total_bytes: u64 = get_objects_resp.objects.iter().map(|o| o.attachment.size).sum();

                sender.start_stage("Exporting".to_owned(), vec!["Cleaning Up".to_owned()]);

                let mut objs_done: usize = 0;
                let mut bytes_done: u64 = 0;

                for obj in get_objects_resp.objects
                {
                    sender.set(100.0 * (bytes_done as f64) / (total_bytes as f64),
                        vec![
                            format!("{} of {} objects", objs_done, num_objects_resp.num_objects),
                            format!("{} of {} of media", format::bytes_to_string(bytes_done), format::bytes_to_string(total_bytes))]);

                    objs_done += 1;
                    bytes_done += obj.attachment.size;

                    let activity_date = obj.activity_time.to_chrono_fixed_offset().date();

                    let path = vec![
                        format!("{}", activity_date.year()),
                        format!("{:02}", activity_date.month()),
                        format!("{:02}", activity_date.day()),
                    ];

                    let attachment_req = picvudb::msgs::GetAttachmentDataRequest
                    {
                        object_id: obj.id.clone(),
                        specific_hash: None,
                    };

                    let attachment_resp = store.write_transaction(|ops|
                        {
                            attachment_req.execute(ops)
                        })?;

                    if let picvudb::msgs::GetAttachmentDataResponse::Found{ metadata, bytes } = attachment_resp
                    {
                        writer.write_file(
                            &path,
                            &metadata.filename,
                            &bytes)?;

                        let attachment_data = data::AttachmentMetadata
                        {
                            filename: obj.attachment.filename.clone(),
                            created: obj.attachment.created.clone(),
                            modified: obj.attachment.modified.clone(),
                            mime: obj.attachment.mime.to_string(),
                            size: obj.attachment.size.clone(),
                            orientation: obj.attachment.orientation.clone(),
                            dimensions: obj.attachment.dimensions.clone(),
                            duration: obj.attachment.duration.clone(),
                            hash: obj.attachment.hash.clone(),
                        };

                        let tags_data = obj.tags.iter().map(|t|
                        {
                            data::TagMetadata
                            {
                                name: t.name.clone(),
                                kind: t.kind.clone(),
                                rating: t.rating.clone(),
                                censor: t.censor.clone(),
                            }
                        }).collect();

                        let obj_data = data::ObjectMetadata
                        {
                            created_time: obj.created_time.clone(),
                            modified_time: obj.modified_time.clone(),
                            activity_time: obj.activity_time.clone(),
                            title: obj.title.map(|m| m.get_markdown()),
                            notes: obj.notes.map(|m| m.get_markdown()),
                            rating: obj.rating.clone(),
                            censor: obj.censor.clone(),
                            location: obj.location.clone(),
                            attachment: attachment_data,
                            tags: tags_data,
                        };

                        let json_metadata = serde_json::to_string_pretty(&obj_data).unwrap();

                        writer.write_file(
                            &path,
                            &format!("{}.json", metadata.filename),
                            &json_metadata.as_bytes().to_vec())?;
                    }
                    else
                    {
                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not load attachment data").into());
                    }
                }

                let mut results = vec![
                    format!("Exported {} objects", objs_done),
                    format!("Exported {} of media", format::bytes_to_string(bytes_done)),
                ];

                results.append(&mut writer.close_and_summarize()?);

                sender.start_stage("Completed".to_owned(), vec![]);
                sender.set(100.0, results);

                Ok(())
            }).await?;

            Ok(())
        })
    }
}
