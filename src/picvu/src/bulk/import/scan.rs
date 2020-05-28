use std::thread::{spawn, JoinHandle};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryInto;
use flate2::read::GzDecoder;
use tar::Archive;

use crate::bulk::progress::ProgressSender;
use crate::format;

pub struct FileEntry
{
    pub display_path: String,
    pub archive_path: String,
    pub file_name: String,
    pub ext: String,
    pub size: u64,
    pub bytes: Vec<u8>,
    pub percent: f64,
    pub progress_bytes: String,
}

pub struct Scanner
{
    total_bytes: u64,
    file_names: Vec<String>,    
}

impl Scanner
{
    pub fn new(path: String, progress_sender: ProgressSender) -> Result<Self, std::io::Error>
    {
        let (total_bytes, file_names) = initial_scan(path, progress_sender)?;

        Ok(Scanner { total_bytes, file_names })
    }

    pub fn iter<F>(self, needs_file_bytes: F) -> ScanIterator
        where F: Fn(&String) -> bool + 'static + Send
    {
        ScanIterator::new(self.total_bytes, self.file_names, needs_file_bytes)
    }

    pub fn clone_iter<F>(&self, needs_file_bytes: F) -> ScanIterator
        where F: Fn(&String) -> bool + 'static + Send
    {
        ScanIterator::new(self.total_bytes, self.file_names.clone(), needs_file_bytes)
    }
}

pub struct ScanIterator
{
    rx: Option<Receiver<Option<Result<FileEntry, std::io::Error>>>>,
    thread: Option<JoinHandle<()>>,
}

impl ScanIterator
{
    fn new<F>(total_bytes: u64, file_names: Vec<String>, needs_file_bytes: F) -> Self
        where F: Fn(&String) -> bool + 'static + Send
    {
        let (tx, rx) = sync_channel(0);
        let thread = spawn(move ||
            {
                if let Err(e) = full_scan(tx.clone(), total_bytes, file_names, needs_file_bytes)
                {
                    let _ = tx.send(Some(Err(e)));
                }
            });

        ScanIterator { rx: Some(rx), thread: Some(thread) }
    }
}

impl std::ops::Drop for ScanIterator
{
    fn drop(&mut self)
    {
        {
            // Dropping the RX handle
            // will cause the thread to quit early
            let _ = self.rx.take();
        }

        // Wait for the thread to end
        let _ = self.thread.take().unwrap().join();
    }
}

impl std::iter::Iterator for ScanIterator
{
    type Item = Result<FileEntry, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item>
    {
        match &self.rx
        {
            None =>
            {
                // Already ended

                None
            },
            Some(rx) =>
            {
                match rx.recv()
                {
                    Ok(None) =>
                    {
                        // The scan thread has completed - close the iterator
                        // and return None

                        let _ = self.rx.take();

                        None
                    },
                    Ok(Some(entry)) =>
                    {
                        Some(entry)
                    },
                    Err(e) =>
                    {
                        // The scan thread has aborted - close the iterator
                        // and return the converted error.

                        let _ = self.rx.take();

                        Some(Err(std::io::Error::new(std::io::ErrorKind::Interrupted, format!("Scan thread interrupted: {:?}", e))))
                    },
                }
            },
        }
    }
}

fn initial_scan(path: String, progress_sender: ProgressSender) -> Result<(u64, Vec<String>), std::io::Error>
{
    let mut total_bytes: u64 = 0;
    let mut file_names: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(path)?
    {
        let entry = entry?;

        let path = entry.path().as_path()
            .to_str()
            .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Path {} contains non-UTF-8 bytes", entry.path().display())))?
            .to_owned();

        progress_sender.set(0.0, vec![path.clone()]);

        if entry.file_type()?.is_dir()
        {
            // TODO - recurse into dirs
        }
        else if entry.file_type()?.is_file()
        {
            let metadata = entry.metadata()?;

            total_bytes += metadata.len();
            file_names.push(path);
        }
    }

    Ok((total_bytes, file_names))
}

fn full_scan<F>(tx: SyncSender<Option<Result<FileEntry, std::io::Error>>>, total_bytes: u64, file_names: Vec<String>, needs_file_bytes: F) -> Result<(), std::io::Error>
    where F: Fn(&String) -> bool + 'static + Send
{
    let mut bytes_processed = 0;

    for file_name in file_names
    {
        let file_size = std::fs::metadata(file_name.clone())?.len();

        if file_name.ends_with(".tar.gz")
            || file_name.ends_with(".tgz")
        {
            let tar_gz = File::open(file_name.clone())?;
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

                let entry_display_path = format!("{} => {}", file_name, path_str);
                let entry_archive_path = path_str.clone();
                let entry_file_name = Path::new(&path_str).file_name().unwrap_or_default().to_str().unwrap().to_owned();
                let entry_ext = Path::new(&path_str).extension().unwrap_or_default().to_str().unwrap().to_owned().to_ascii_lowercase();

                let tar_bytes_read = counted_get.get();

                let entry_file_size: usize = entry.header().size()?
                    .try_into()
                    .map_err(|_| {std::io::Error::new(std::io::ErrorKind::InvalidData, format!("File {} is too large", path_str))})?;

                let mut bytes = Vec::new();

                if needs_file_bytes(&entry_file_name)
                {
                    bytes.reserve(entry_file_size);
                    entry.read_to_end(&mut bytes)?;
                }

                let percent_bytes = bytes_processed + tar_bytes_read;
                let percent = (percent_bytes as f64) / (total_bytes as f64) * 100.0;
                let progress_bytes = format!("Processed {} of {}",
                    format::bytes_to_string(percent_bytes),
                    format::bytes_to_string(total_bytes));

                let result = FileEntry
                {
                    display_path: entry_display_path,
                    archive_path: entry_archive_path,
                    file_name: entry_file_name,
                    ext: entry_ext,
                    size: entry_file_size as u64,
                    bytes: bytes,
                    percent: percent,
                    progress_bytes: progress_bytes,
                };

                if tx.send(Some(Ok(result))).is_err()
                {
                    // The iterator has been dropped - we should abort
                    return Ok(())
                }
            }

            bytes_processed += file_size;
        }
    }

    // Send through a None to indicate that we're done

    let _ = tx.send(None);

    Ok(())
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