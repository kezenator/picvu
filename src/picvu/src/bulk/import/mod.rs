use std::future::Future;
use std::pin::Pin;
use actix_web::web;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use flate2::read::GzDecoder;
use tar::Archive;

use picvudb::StoreAccess;
use picvudb::ApiMessage;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;

mod error;
mod metadata;

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
                    vec!["Loading Metadata".to_owned(), "Importing Media".to_owned()]);

                let metadata = std::fs::metadata(self.tar_gz_file_path.clone())?;
                let len = metadata.len();

                let mut path_to_info: HashMap<String, (String, mime::Mime)> = HashMap::new();

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
                        let progress_cur_file = path.display().to_string();
                        let progress_bytes = format!("Processed {} of {} bytes", bytes_read, len);

                        sender.set(percent, vec![progress_cur_file, progress_bytes]);

                        let ext = path.extension().unwrap_or_default().to_str().unwrap().to_owned().to_ascii_lowercase();

                        let mut insert = |mime: &str| -> Result<(), ImportError>
                        {
                            let mime = mime.parse::<mime::Mime>()
                                .map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid MIME type: {:?}", e)) })?;

                            let file_name = path
                                .file_name()
                                .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Contained file name has no file name"))?
                                .to_str()
                                .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Contained file name contains non-UTF-8 byte sequences"))?
                                .to_owned();

                            path_to_info.insert(path_str.clone(), (file_name, mime));

                            Ok(())
                        };

                        if (ext == "jpg")
                            || (ext == "jpeg")
                        {
                            insert("image/jpeg")?;
                        }
                        else if ext == "png"
                        {
                            insert("image/png")?;
                        }
                        else if ext == "gif"
                        {
                            insert("image/gif")?;
                        }
                        else if ext == "mp4"
                        {
                            // TODO - skipping video files for now
                            //insert("video/mp4")?;
                        }
                        else if ext == "json"
                        {
                            // Ignore JSON metadata files
                        }
                        else if path_str == "Takeout/archive_browser.html"
                        {
                            // Ignore Google Takeout browsing file
                        }
                        else
                        {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unhandled extension for file {}", path_str)).into());
                        }
                    }
                }

                sender.start_stage(
                    "Loading Metadata".to_owned(),
                    vec!["Importing Media".to_owned()]);

                let mut path_to_metadata: HashMap<String, metadata::Metadata> = HashMap::new();

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
                        let progress_bytes = format!("Processed {} of {} bytes", bytes_read, len);
                        let progress_files = format!("Found metadata for {} out of {} media files", path_to_metadata.len(), path_to_info.len());

                        sender.set(percent, vec![progress_cur_file, progress_bytes, progress_files]);

                        if path_str.ends_with(".json")
                        {
                            let media_name = path_str[0..(path_str.len()-5)].to_owned();

                            if path_to_info.contains_key(&media_name)
                            {
                                let mut json = String::new();
                                entry.read_to_string(&mut json)?;

                                let metadata = serde_json::from_str::<metadata::Metadata>(&json)
                                    .map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Could not decode metadata JSON file {}: {:?}", path_str, e)) })?;

                                path_to_metadata.insert(media_name, metadata);
                            }
                        }
                    }
                }

                sender.start_stage(
                    "Importing Media".to_owned(),
                    vec![]);

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
                        let progress_bytes = format!("Processed {} of {} bytes", bytes_read, len);

                        sender.set(percent, vec![progress_cur_file, progress_bytes]);

                        if let Some((file_name, mime)) = path_to_info.get(&path_str)
                        {
                            let file_size: usize = entry.header().size()?
                                .try_into()
                                .map_err(|_| {std::io::Error::new(std::io::ErrorKind::InvalidData, format!("File {} is too large", path_str))})?;

                            let mut bytes = Vec::new();
                            bytes.reserve(file_size);

                            entry.read_to_end(&mut bytes)?;

                            let data = picvudb::data::add::ObjectData
                            {
                                title: Some(file_name.clone()),
                                additional: picvudb::data::add::AdditionalData::Photo
                                {
                                    attachment: picvudb::data::add::Attachment{
                                        filename: file_name.clone(),
                                        created: picvudb::data::Date::now(),
                                        modified: picvudb::data::Date::now(),
                                        mime: mime.clone(),
                                        bytes: bytes,
                                    },
                                },                            
                            };

                            let msg = picvudb::msgs::AddObjectRequest{ data };

                            store.write_transaction(|ops|
                            {
                                msg.execute(ops)
                            })?;
                        }
                    }
                }

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