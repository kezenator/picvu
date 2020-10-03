use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

pub trait ExportWriter
{
    fn write_file(&mut self, path: &Vec<String>, filename: &String, bytes: &Vec<u8>) -> Result<(), std::io::Error>;
    fn close_and_summarize(self) -> Result<Vec<String>, std::io::Error>;
}

pub struct FileExportWriter
{
    folder: String,
    written_files: HashMap<PathBuf, HashSet<OsString>>,
    new_count: FileCounter,
    update_count: FileCounter,
    unchanged_count: FileCounter,
    deleted_files: u64,
    deleted_folders: u64,
}

impl FileExportWriter
{
    pub fn new(folder: String) -> Result<Self, std::io::Error>
    {
        Ok(FileExportWriter
        {
            folder: folder,
            written_files: HashMap::new(),
            new_count: FileCounter::new(),
            update_count: FileCounter::new(),
            unchanged_count: FileCounter::new(),
            deleted_files: 0,
            deleted_folders: 0,
        })
    }
}

impl ExportWriter for FileExportWriter
{
    fn write_file(&mut self, path: &Vec<String>, filename: &String, bytes: &Vec<u8>) -> Result<(), std::io::Error>
    {
        let mut full_path = std::path::Path::new(&self.folder).to_path_buf();
        
        for p in path
        {
            self.written_files.entry(full_path.clone()).or_default().insert(p.into());

            full_path = full_path.join(p);

            if !full_path.is_dir()
            {
                std::fs::create_dir(&full_path)?;
            }
        }
        
        {
            let entry = self.written_files.entry(full_path.clone()).or_default();
            
            if !entry.insert(filename.into())
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Duplicate files with name {}", full_path.join(filename).display())));
            }
        }

        full_path = full_path.join(filename);

        let write = match check_file(&full_path, bytes)
        {
            CheckResult::New =>
            {
                self.new_count.count(bytes.len());
                true
            },
            CheckResult::Update =>
            {
                self.update_count.count(bytes.len());
                true
            },
            CheckResult::NoChange =>
            {
                self.unchanged_count.count(bytes.len());
                false
            },
        };

        if write
        {
            let mut file = std::fs::File::create(full_path)?;
            file.write_all(bytes)?;
        }

        Ok(())
    }

    fn close_and_summarize(mut self) -> Result<Vec<String>, std::io::Error>
    {
        for written_entry in self.written_files
        {
            for file_entry in std::fs::read_dir(written_entry.0)?
            {
                let file_entry = file_entry?;

                if !written_entry.1.contains(&file_entry.file_name())
                {
                    if file_entry.file_type()?.is_dir()
                    {
                        self.deleted_folders += 1;
                        std::fs::remove_dir_all(file_entry.path())?;
                    }
                    else
                    {
                        self.deleted_files += 1;
                        std::fs::remove_file(file_entry.path())?;
                    }
                }
            }
        }

        Ok(vec![
            self.new_count.summarize("New"),
            self.update_count.summarize("Updated"),
            self.unchanged_count.summarize("Unchanged"),
            format!("Deleted: {} files, {} folders", self.deleted_files, self.deleted_folders),
        ])
    }
}

enum CheckResult
{
    New,
    Update,
    NoChange,
}

fn check_file(path: &PathBuf, contents: &Vec<u8>) -> CheckResult
{
    match check_file_internal(path, contents)
    {
        Ok(result) => result,
        Err(_) => CheckResult::New,
    }
}

fn check_file_internal(path: &PathBuf, contents: &Vec<u8>) -> Result<CheckResult, std::io::Error>
{
    let metadata = std::fs::metadata(path)?;

    if !metadata.is_file()
    {
        return Ok(CheckResult::New);
    }

    if metadata.len() != (contents.len() as u64)
    {
        return Ok(CheckResult::Update);
    }

    let existing = std::fs::read(path)?;

    if existing != *contents
    {
        return Ok(CheckResult::Update);
    }

    Ok(CheckResult::NoChange)
}

struct FileCounter
{
    file_count: u64,
    byte_count: u64,
}

impl FileCounter
{
    pub fn new() -> Self
    {
        FileCounter{ file_count: 0, byte_count: 0 }
    }

    pub fn count(&mut self, bytes: usize)
    {
        self.file_count += 1;
        self.byte_count += bytes as u64;
    }

    pub fn summarize(&self, name: &str) -> String
    {
        format!("{}: {} files, {}", name, self.file_count, crate::format::bytes_to_string(self.byte_count))
    }
}