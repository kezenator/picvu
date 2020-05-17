use std::collections::HashMap;

use crate::err::Error;
use crate::models::*;
use crate::data;

pub trait ReadOps
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>;
    fn get_all_objects(&self) -> Result<Vec<Object>, Error>;
    fn get_attachment_metadata(&self, obj_id: &String) -> Result<Option<AttachmentMetadata>, Error>;
    fn get_attachment_data(&self, obj_id: &String) -> Result<Option<Vec<u8>>, Error>;
}

pub trait WriteOps: ReadOps
{
    fn add_object(&self, title: Option<String>) -> Result<Object, Error>;
    fn add_attachment(&self, obj_id: &String, filename: String, created: data::Date, modified: data::Date, mime: String, bytes: Vec<u8>) -> Result<(), Error>;
}
