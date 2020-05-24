use std::future::Future;
use std::pin::Pin;
use actix_web::web;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;
use crate::format;
use crate::analyse;

mod error;

use error::*;

pub struct FolderImport
{
    tar_gz_file_path: String,
    db_uri: String,
}

impl FolderImport
{
    pub fn new(tar_gz_file_path: String, db_uri: String) -> Self
    {
        FolderImport
        {
            tar_gz_file_path,
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
        format!("Bulk Import: {}", self.tar_gz_file_path)
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        Box::pin(async move
        {
            web::block(move || -> Result<(), ImportError>
            {
                sender.start_stage(
                    "Scanning file".to_owned(),
                    vec!["Loading Metadata".to_owned(), "Importing Media".to_owned(), "Summary".to_owned()]);

                let metadata = std::fs::metadata(self.tar_gz_file_path.clone())?;
                let len = metadata.len();

                let mut path_to_info: HashMap<String, (String, mime::Mime)> = HashMap::new();
                let mut is_google_photos_takeout_archive = false;
                let mut warnings: Vec<String> = Vec::new();

                {
                    let tar_gz = File::open(self.tar_gz_file_path.clone())?;
                    let (counted_reader, counted_get) = CountedRead::new(tar_gz);
                    let tar = GzDecoder::new(counted_reader);
                    let mut archive = Archive::new(tar);

                    for entry in archive.entries()?
                    {
                        let entry = entry?;

                        let path = entry.path()?;
                        let path_str = path
                            .to_str()
                            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Contained file name contains non-UTF-8 byte sequences"))?
                            .to_owned();

                        let bytes_read = counted_get.get();

                        let percent = (bytes_read as f64) / (len as f64) * 100.0;
                        let progress_cur_file = path_str.clone();
                        let progress_bytes = format!("Processed {} of {}", format::bytes_to_str(bytes_read), format::bytes_to_str(len));

                        sender.set(percent, vec![progress_cur_file, progress_bytes]);

                        let filename = Path::new(&path_str).file_name().unwrap_or_default().to_str().unwrap().to_owned();
                        let ext = Path::new(&path_str).extension().unwrap_or_default().to_str().unwrap().to_owned().to_ascii_lowercase();

                        if let Some(mime) = analyse::import::guess_mime_type_from_filename(&filename)
                        {
                            path_to_info.insert(path_str.clone(), (filename, mime));
                        }
                        else if ext == "json"
                        {
                            // Ignore JSON metadata files
                        }
                        else if path_str == "Takeout/archive_browser.html"
                        {
                            // If it has the Google Photos Takout browser file, then
                            // assume it's an archive from this service.

                            is_google_photos_takeout_archive = true;
                        }
                        else
                        {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unsupported file: {}", path_str)).into());
                        }
                    }
                }

                sender.start_stage(
                    "Loading Metadata".to_owned(),
                    vec!["Importing Media".to_owned(), "Summary".to_owned()]);

                let mut path_to_metadata: HashMap<String, analyse::takeout::Metadata> = HashMap::new();

                if is_google_photos_takeout_archive
                {
                    let tar_gz = File::open(self.tar_gz_file_path.clone())?;
                    let (counted_reader, counted_get) = CountedRead::new(tar_gz);
                    let tar = GzDecoder::new(counted_reader);
                    let mut archive = Archive::new(tar);

                    for entry in archive.entries()?
                    {
                        let mut entry = entry?;

                        let path = entry.path()?;
                        let path_str = path
                            .to_str()
                            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Contained file name contains non-UTF-8 byte sequences"))?
                            .to_owned();

                        let bytes_read = counted_get.get();

                        let percent = (bytes_read as f64) / (len as f64) * 100.0;
                        let progress_cur_file = path.display().to_string();
                        let progress_bytes = format!("Processed {} of {}", format::bytes_to_str(bytes_read), format::bytes_to_str(len));
                        let progress_files = format!("Found metadata for {} out of {} media files", path_to_metadata.len(), path_to_info.len());

                        sender.set(percent, vec![progress_cur_file, progress_bytes, progress_files]);

                        if path_str.ends_with(".json")
                        {
                            let media_name = path_str[0..(path_str.len()-5)].to_owned();

                            if path_to_info.contains_key(&media_name)
                            {
                                let mut json_bytes = Vec::new();
                                entry.read_to_end(&mut json_bytes)?;

                                let metadata = analyse::takeout::parse_google_photos_takeout_metadata(json_bytes, &path_str)?;

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

                let mut summary_media_files: usize = 0;
                let mut summary_media_bytes: u64 = 0;
                let mut summary_with_google_metadata: usize = 0;
                let mut summary_with_location: usize = 0;

                let store = picvudb::Store::new(&self.db_uri)?;

                {
                    let tar_gz = File::open(self.tar_gz_file_path.clone())?;
                    let (counted_reader, counted_get) = CountedRead::new(tar_gz);
                    let tar = GzDecoder::new(counted_reader);
                    let mut archive = Archive::new(tar);

                    for entry in archive.entries()?
                    {
                        let mut entry = entry?;

                        let path = entry.path()?;
                        let path_str = path
                            .to_str()
                            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Contained file name contains non-UTF-8 byte sequences"))?
                            .to_owned();

                        let bytes_read = counted_get.get();

                        let percent = (bytes_read as f64) / (len as f64) * 100.0;
                        let progress_cur_file = path.display().to_string();
                        let progress_bytes = format!("Processed {} of {}", format::bytes_to_str(bytes_read), format::bytes_to_str(len));
                        let progress_files = format!("Processed {} of {} media files", summary_media_files, path_to_info.len());

                        sender.set(percent, vec![progress_cur_file, progress_bytes, progress_files]);

                        if let Some((file_name, _mime)) = path_to_info.get(&path_str)
                        {
                            let file_size: usize = entry.header().size()?
                                .try_into()
                                .map_err(|_| {std::io::Error::new(std::io::ErrorKind::InvalidData, format!("File {} is too large", path_str))})?;

                            let mut bytes = Vec::new();
                            bytes.reserve(file_size);

                            entry.read_to_end(&mut bytes)?;

                            summary_media_files += 1;
                            summary_media_bytes += bytes.len() as u64;

                            let google_metadata = path_to_metadata.remove(&path_str);

                            if google_metadata.is_some()
                            {
                                summary_with_google_metadata += 1;                                
                            }
                            
                            let add_msg = analyse::import::create_add_object_for_import(
                                bytes,
                                file_name,
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
                    format!("Imported {} of media data", format::bytes_to_str(summary_media_bytes)),
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

struct CountedGet
{
    count: Rc<RefCell<u64>>,
}

impl CountedGet
{
    pub fn get(&self) -> u64
    {
        *self.count.borrow()
    }
}

struct CountedRead<SubReader>
    where SubReader: Read
{
    sub_reader: SubReader,
    count: Rc<RefCell<u64>>,
}

impl<SubReader> CountedRead<SubReader>
    where SubReader: Read
{
    pub fn new(sub_reader: SubReader) -> (Self, CountedGet)
    {
        let count = Rc::new(RefCell::new(0));
        (CountedRead{ sub_reader, count: count.clone() }, CountedGet{ count: count.clone() })
    }
}

impl<SubReader> Read for CountedRead<SubReader>
    where SubReader: Read
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
    {
        let result = self.sub_reader.read(buf);

        if let Ok(count) = result
        {
            *self.count.borrow_mut() += count as u64;
        }

        return result;
    }
}