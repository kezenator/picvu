use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use actix_web::web;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use googlephotos::auth::AccessToken;

use crate::bulk::BulkOperation;
use crate::bulk::export;
use crate::bulk::progress::ProgressSender;
use crate::bulk::sync::mediaitems::MediaItemDatabase;
use crate::format;
use crate::analyse;
use crate::analyse::warning::{Warning, WarningKind};

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
    google_api_key: String,
    access_token: AccessToken,
    import_options: analyse::import::ImportOptions,
}

impl FolderImport
{
    pub fn new(folder_path: String, db_uri: String, google_api_key: String, access_token: AccessToken, import_options: analyse::import::ImportOptions) -> Self
    {
        FolderImport
        {
            folder_path,
            db_uri,
            google_api_key,
            access_token,
            import_options,
        }
    }
}

impl BulkOperation for FolderImport
{
    type Error = actix_web::error::BlockingError<ImportError>;
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
                let mut is_picvu_export_archive = false;
                let mut warnings: Vec<Warning> = Vec::new();

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
                        else if entry.file_name == "picvu.export.json"
                        {
                            // It is an export from this application

                            is_picvu_export_archive = true;
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

                // Ensure we only have one supported set of metadata

                if is_picvu_export_archive && is_google_photos_takeout_archive
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Import path appears to have both Google Takeout and Picvu Export metadata - only one is supported at a time").into());
                }

                let google_photos_db =
                {
                    if is_google_photos_takeout_archive
                    {
                        sender.start_stage(
                            "Loading Google Photots Metadata".to_owned(),
                            vec!["Loading Metadata".to_owned(), "Importing Media".to_owned(), "Summary".to_owned()]);

                        MediaItemDatabase::load_all(&self.access_token, &sender)?
                    }
                    else
                    {
                        MediaItemDatabase::empty()
                    }
                };

                // Add warnings for Google Photos entries with duplicate file names
                // as we can't be sure we get the right one

                {
                    for (filename, len) in google_photos_db.filenames_with_multiple_media_items()
                    {
                        warnings.push(Warning::new(
                            filename,
                            WarningKind::DuplicateGooglePhotosFilename,
                            format!("{} items have the same filename", len)
                        ));
                    }
                }

                // Load all of the metadata

                sender.start_stage(
                    "Loading Metadata".to_owned(),
                    vec!["Importing Media".to_owned(), "Summary".to_owned()]);

                let mut path_to_picvu_metadata: HashMap<String, export::data::ObjectMetadata> = HashMap::new();
                let mut path_to_google_metadata: HashMap<String, analyse::takeout::Metadata> = HashMap::new();

                if is_picvu_export_archive || is_google_photos_takeout_archive
                {
                    for entry in scanner.clone_iter(|file_name| { file_name.ends_with(".json") })
                    {
                        let entry = entry?;

                        let sum = path_to_picvu_metadata.len() + path_to_google_metadata.len();

                        let progress_files = format!("Found metadata for {} out of {} media files", sum, path_to_info.len());

                        sender.set(entry.percent, vec![entry.display_path.clone(), entry.progress_bytes, progress_files]);

                        if entry.archive_path.ends_with(".json")
                        {
                            let media_name = entry.archive_path[0..(entry.archive_path.len()-5)].to_owned();

                            if path_to_info.contains_key(&media_name)
                            {
                                if is_picvu_export_archive
                                {
                                    let metadata = export::data::parse_object_metadata(entry.bytes, &entry.display_path)?;

                                    path_to_picvu_metadata.insert(media_name, metadata);
                                }
                                else if is_google_photos_takeout_archive
                                {
                                    let metadata = analyse::takeout::parse_google_photos_takeout_metadata(entry.bytes, &entry.display_path)?;

                                    path_to_google_metadata.insert(media_name, metadata);
                                }
                            }
                        }
                    }
                }

                sender.start_stage(
                    "Importing Media".to_owned(),
                    vec!["Summary".to_owned()]);

                let google_cache = analyse::google::GoogleCache::new(self.google_api_key);

                let num_found_metadata_files = path_to_picvu_metadata.len() + path_to_google_metadata.len();

                let mut summary_imported_media_files: usize = 0;
                let mut summary_imported_media_bytes: u64 = 0;
                let mut summary_with_picvu_metadata: usize = 0;
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
                        let progress_picvu_metadata = format!("Processed {} of {} Picvu metadata files", summary_with_picvu_metadata, num_found_metadata_files);
                        let progress_google_metadata = format!("Processed {} of {} Google Takeout metadata files", summary_with_google_metadata, num_found_metadata_files);
                        let progress_location = format!("Processed {} files with location data", summary_with_location);
                        let progress_skipped_files = format!("Skipped {} media files", summary_skipped_media_files);

                        sender.set(entry.percent, vec![entry.display_path, entry.progress_bytes,
                            progress_files, progress_imported_files, progress_imported_bytes,
                            progress_picvu_metadata, progress_google_metadata, progress_location, progress_skipped_files]);

                        if let Some(found_info) = path_to_info.get(&entry.archive_path)
                        {
                            let picvu_metadata = path_to_picvu_metadata.get(&entry.archive_path).cloned();
                            let google_metadata = path_to_google_metadata.get(&entry.archive_path).cloned();

                            if picvu_metadata.is_some()
                            {
                                summary_with_picvu_metadata += 1;
                            }

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
                                        if let Some(_other_metadata) = path_to_google_metadata.get(&other_path)
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

                                                warnings.push(Warning::new(
                                                    found_info.file_name.clone(),
                                                    WarningKind::SkippedDuplicateMvImgPart,
                                                    "Skipped the MP4 part of a moving image".to_owned()));
                                            }
                                        }
                                    }
                                }

                                if !skip
                                {
                                    warnings.push(Warning::new(
                                        found_info.file_name.clone(),
                                        WarningKind::NoGoogleTakeoutMetadataAvailable,
                                        "No Google Takeout metadata found".to_owned()));
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

                                let ext_ref = google_photos_db.find_best_match(&entry.file_name, &entry.created);

                                if is_google_photos_takeout_archive
                                    && ext_ref.is_none()
                                {
                                    warnings.push(Warning::new(
                                        entry.file_name.clone(),
                                        WarningKind::MissingGooglePhotosReference,
                                        "No Google Photos link found".to_owned()));
                                }

                                let add_msg = analyse::import::create_add_object_for_import(
                                    entry.bytes,
                                    &entry.file_name,
                                    &google_cache,
                                    &import_options,
                                    entry.created,
                                    Some(entry.modified),
                                    picvu_metadata,
                                    google_metadata,
                                    ext_ref,
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
                    format!("{} files had Picvu metadata", summary_with_picvu_metadata),
                    format!("{} files had Google Photos Takeout metadata", summary_with_google_metadata),
                    format!("{} files had location data", summary_with_location),
                    format!("Skipped {} media files", summary_skipped_media_files),
                ];

                if !warnings.is_empty()
                {
                    status.push(format!("{} Warnings:", warnings.len()));

                    warnings.sort();

                    for w in warnings
                    {
                        status.push(format!("{:?}", w));
                    }
                }

                sender.set(100.0, status);

                Ok(())
            }).await?;

            Ok(())
        })
    }
}
