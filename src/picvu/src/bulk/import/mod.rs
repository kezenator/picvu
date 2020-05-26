use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use actix_web::web;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;
use crate::format;
use crate::analyse;

mod error;
mod scan;

use error::*;

pub struct FolderImport
{
    folder_path: String,
    db_uri: String,
}

impl FolderImport
{
    pub fn new(folder_path: String, db_uri: String) -> Self
    {
        FolderImport
        {
            folder_path,
            db_uri,
        }
    }
}

impl BulkOperation for FolderImport
{
    type Error = actix_rt::blocking::BlockingError<ImportError>;
    type Future = Pin<Box<dyn Future<Output=Result<(), Self::Error>>>>;

    fn name(&self) -> String
    {
        format!("Bulk Import: {}", self.folder_path)
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        Box::pin(async move
        {
            web::block(move || -> Result<(), ImportError>
            {
                sender.start_stage(
                    "Finding files".to_owned(),
                    vec!["Scanning files and archives".to_owned(), "Loading Metadata".to_owned(), "Importing Media".to_owned(), "Summary".to_owned()]);

                let scanner = scan::Scanner::new(self.folder_path, sender.clone())?;

                sender.start_stage(
                    "Scanning files and archives".to_owned(),
                    vec!["Loading Metadata".to_owned(), "Importing Media".to_owned(), "Summary".to_owned()]);

                let mut path_to_info: HashMap<String, (String, mime::Mime)> = HashMap::new();
                let mut is_google_photos_takeout_archive = false;
                let mut warnings: Vec<String> = Vec::new();

                {
                    for entry in scanner.clone_iter(|_| { false })
                    {
                        let entry = entry?;

                        sender.set(entry.percent, vec![entry.display_path.clone(), entry.progress_bytes]);

                        if let Some(mime) = analyse::import::guess_mime_type_from_filename(&entry.file_name)
                        {
                            path_to_info.insert(entry.archive_path.clone(), (entry.file_name, mime));
                        }
                        else if entry.ext == "json"
                        {
                            // Ignore JSON metadata files
                        }
                        else if entry.archive_path == "Takeout/archive_browser.html"
                        {
                            // If it has the Google Photos Takout browser file, then
                            // assume it's an archive from this service.

                            is_google_photos_takeout_archive = true;
                        }
                        else
                        {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unsupported file: {}", entry.display_path)).into());
                        }
                    }
                }

                sender.start_stage(
                    "Loading Metadata".to_owned(),
                    vec!["Importing Media".to_owned(), "Summary".to_owned()]);

                let mut path_to_metadata: HashMap<String, analyse::takeout::Metadata> = HashMap::new();

                if is_google_photos_takeout_archive
                {
                    for entry in scanner.clone_iter(|file_name| { file_name.ends_with(".json") })
                    {
                        let entry = entry?;

                        let progress_files = format!("Found metadata for {} out of {} media files", path_to_metadata.len(), path_to_info.len());

                        sender.set(entry.percent, vec![entry.display_path.clone(), entry.progress_bytes, progress_files]);

                        if entry.archive_path.ends_with(".json")
                        {
                            let media_name = entry.archive_path[0..(entry.archive_path.len()-5)].to_owned();

                            if path_to_info.contains_key(&media_name)
                            {
                                let metadata = analyse::takeout::parse_google_photos_takeout_metadata(entry.bytes, &entry.display_path)?;

                                path_to_metadata.insert(media_name, metadata);
                            }
                        }
                    }

                    // Check that every media file has associated metadata

                    // TODO - add back in when we support scanning all
                    // archives of the Takeout.

                    // for media_path in path_to_info.keys()
                    // {
                    //     if !path_to_metadata.contains_key(media_path)
                    //     {
                    //         return Err(std::io::Error::new(
                    //             std::io::ErrorKind::InvalidData,
                    //             format!("File {} doesn't contain Google Photos Takeout metadata", media_path)).into());
                    //     }
                    // }
                }

                sender.start_stage(
                    "Importing Media".to_owned(),
                    vec!["Summary".to_owned()]);

                let num_found_metadata_files = path_to_metadata.len();

                let mut summary_media_files: usize = 0;
                let mut summary_media_bytes: u64 = 0;
                let mut summary_with_google_metadata: usize = 0;
                let mut summary_with_location: usize = 0;

                let store = picvudb::Store::new(&self.db_uri)?;

                {
                    for entry in scanner.iter(|_| { true })
                    {
                        let entry = entry?;

                        let progress_files = format!("Processed {} of {} media files", summary_media_files, path_to_info.len());
                        let progress_media_bytes = format!("Imported {} of media data", format::bytes_to_string(summary_media_bytes));
                        let progress_metadata = format!("Processed {} of {} metadata files", summary_with_google_metadata, num_found_metadata_files);
                        let progress_location = format!("Processed {} files with location data", summary_with_location);

                        sender.set(entry.percent, vec![entry.display_path, entry.progress_bytes,
                            progress_files, progress_media_bytes,
                            progress_metadata, progress_location]);

                        if let Some((_file_name, _mime)) = path_to_info.get(&entry.archive_path)
                        {
                            summary_media_files += 1;
                            summary_media_bytes += entry.bytes.len() as u64;

                            let google_metadata = path_to_metadata.remove(&entry.archive_path);

                            if google_metadata.is_some()
                            {
                                summary_with_google_metadata += 1;                                
                            }
                            
                            let add_msg = analyse::import::create_add_object_for_import(
                                entry.bytes,
                                &entry.file_name,
                                None,
                                None,
                                google_metadata,
                                &mut warnings)?;

                            if add_msg.data.location.is_some()
                            {
                                summary_with_location += 1;
                            }

                            store.write_transaction(|ops|
                            {
                                add_msg.execute(ops)
                            })?;
                        }
                    }
                }

                sender.start_stage(
                    "Summary".to_owned(),
                    vec![]);

                let mut status = vec![
                    format!("Imported {} media files", summary_media_files),
                    format!("Imported {} of media data", format::bytes_to_string(summary_media_bytes)),
                    format!("{} files had Google Photos Takeout metadata", summary_with_google_metadata),
                    format!("{} files had location data", summary_with_location),
                ];

                if !warnings.is_empty()
                {
                    status.push(format!("{} Warnings:", warnings.len()));
                    status.append(&mut warnings);
                }

                sender.set(100.0, status);

                Ok(())
            }).await?;

            Ok(())
        })
    }
}
