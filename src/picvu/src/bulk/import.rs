use std::future::Future;
use std::pin::Pin;
use actix_web::web;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use flate2::read::GzDecoder;
use tar::Archive;
use horrorshow::html;

use crate::bulk::BulkOperation;
use crate::bulk::progress::ProgressSender;

pub struct FolderImport
{
    tar_gz_file_path: String,
}

impl FolderImport
{
    pub fn new(tar_gz_file_path: &String) -> Self
    {
        FolderImport
        {
            tar_gz_file_path: tar_gz_file_path.clone(),
        }
    }
}

impl BulkOperation for FolderImport
{
    type Error = actix_rt::blocking::BlockingError<std::io::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<(), Self::Error>>>>;

    fn name(&self) -> String
    {
        format!("Bulk Import: {}", self.tar_gz_file_path)
    }

    fn start(self, sender: ProgressSender) -> Self::Future
    {
        Box::pin(async move
        {
            web::block(move || -> Result<(), std::io::Error>
            {
                let metadata = std::fs::metadata(self.tar_gz_file_path.clone())?;
                let len = metadata.len();

                let tar_gz = File::open(self.tar_gz_file_path.clone())?;
                let (counted_reader, counted_get) = CountedRead::new(tar_gz);
                let tar = GzDecoder::new(counted_reader);
                let mut archive = Archive::new(tar);

                for entry in archive.entries()?
                {
                    let entry = entry?;

                    let path = entry.path()?;

                    let bytes_read = counted_get.get();

                    let percent = (bytes_read as f64) / (len as f64) * 100.0;

                    sender.set(html!
                    {
                        p : (path.display().to_string());
                        p : (format!("{:.2} ({} of {})", percent, bytes_read, len))
                    });

                    println!("{}", path.display());
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