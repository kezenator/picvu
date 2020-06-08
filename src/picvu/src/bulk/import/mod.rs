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

struct FoundMediaFileInfo
{
    file_name: String,
    size: u64,
}

pub struct FolderImport
{
    folder_path: String,
    db_uri: String,
    import_options: analyse::import::ImportOptions,
}

impl FolderImport
{
    pub fn new(folder_path: String, db_uri: String, import_options: analyse::import::ImportOptions) -> Self
    {
        FolderImport
        {
            folder_path,
            db_uri,
            import_options,
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

                let mut path_to_info: HashMap<String, FoundMediaFileInfo> = HashMap::new();
                let mut is_google_photos_takeout_archive = false;
                let mut warnings: Vec<String> = Vec::new();

                {
                    for entry in scanner.clone_iter(|_| { false })
                    {
                        let entry = entry?;

                        sender.set(entry.percent, vec![entry.display_path.clone(), entry.progress_bytes]);

                        if let Some(_mime_type) = analyse::import::guess_mime_type_from_filename(&entry.file_name)
                        {
                            let info = FoundMediaFileInfo
                            {
                                file_name: entry.file_name,
                                size: entry.size,
                            };

                            path_to_info.insert(entry.archive_path.clone(), info);
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
                }

                sender.start_stage(
                    "Importing Media".to_owned(),
                    vec!["Summary".to_owned()]);

                let num_found_metadata_files = path_to_metadata.len();

                let mut summary_imported_media_files: usize = 0;
                let mut summary_imported_media_bytes: u64 = 0;
                let mut summary_with_google_metadata: usize = 0;
                let mut summary_with_location: usize = 0;
                let mut summary_skipped_media_files: usize = 0;

                let store = picvudb::Store::new(&self.db_uri)?;
                let import_options = self.import_options.clone();

                {
                    for entry in scanner.iter(|_| { true })
                    {
                        let entry = entry?;

                        let progress_files = format!("Processed {} of {} media files", (summary_imported_media_files + summary_skipped_media_files), path_to_info.len());
                        let progress_imported_files = format!("Imported {} media files", summary_imported_media_files);
                        let progress_imported_bytes = format!("Imported {} of media data", format::bytes_to_string(summary_imported_media_bytes));
                        let progress_metadata = format!("Processed {} of {} metadata files", summary_with_google_metadata, num_found_metadata_files);
                        let progress_location = format!("Processed {} files with location data", summary_with_location);
                        let progress_skipped_files = format!("Skipped {} media files", summary_skipped_media_files);

                        sender.set(entry.percent, vec![entry.display_path, entry.progress_bytes,
                            progress_files, progress_imported_files, progress_imported_bytes,
                            progress_metadata, progress_location, progress_skipped_files]);

                        if let Some(found_info) = path_to_info.get(&entry.archive_path)
                        {
                            let google_metadata = path_to_metadata.get(&entry.archive_path).cloned();

                            if google_metadata.is_some()
                            {
                                summary_with_google_metadata += 1;                                
                            }

                            let mut skip = false;

                            if is_google_photos_takeout_archive
                                && google_metadata.is_none()
                            {
                                if found_info.file_name.starts_with("MVIMG")
                                    && found_info.file_name.ends_with("(1).jpg")
                                    && (analyse::img::MvImgSplit::Mp4Only == analyse::img::parse_mvimg_split(&entry.bytes, &found_info.file_name))
                                {
                                    let archive_path_len = entry.archive_path.len();
                                    let other_path = format!("{}.jpg", &entry.archive_path[0..(archive_path_len - 7)]);

                                    if let Some(other_info) = path_to_info.get(&other_path)
                                    {
                                        if let Some(_other_metadata) = path_to_metadata.get(&other_path)
                                        {
                                            if other_info.size > found_info.size
                                            {
                                                // This file is a "MVIMG....(1).jpg" file, has no metadata,
                                                // is MP4 only (with no JPEG component), but a file
                                                // without the "(1)" suffix exists with metadata, and it's
                                                // size is larger than our size.
                                                //
                                                // Google seems to extract moving images and creates a second
                                                // file with the movie part - but it's already contained in the
                                                // other file.
                                                //
                                                // We should just skip these files

                                                skip = true;
                                                warnings.push(format!("{} was skipped - it's the MP4 part of a moving image", found_info.file_name));
                                            }
                                        }
                                    }
                                }

                                if !skip
                                {
                                    warnings.push(format!("{} has no metadata", found_info.file_name));
                                }
                            }

                            if skip
                            {
                                summary_skipped_media_files += 1;
                            }
                            else
                            {
                                summary_imported_media_files += 1;
                                summary_imported_media_bytes += entry.bytes.len() as u64;

                                let add_msg = analyse::import::create_add_object_for_import(
                                    entry.bytes,
                                    &entry.file_name,
                                    &import_options,
                                    entry.created,
                                    Some(entry.modified),
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
                }

                sender.start_stage(
                    "Summary".to_owned(),
                    vec![]);

                let mut status = vec![
                    format!("Imported {} media files", summary_imported_media_files),
                    format!("Imported {} of media data", format::bytes_to_string(summary_imported_media_bytes)),
                    format!("{} files had Google Photos Takeout metadata", summary_with_google_metadata),
                    format!("{} files had location data", summary_with_location),
                    format!("Skipped {} media files", summary_skipped_media_files),
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
