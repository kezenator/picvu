use std::collections::HashMap;

use crate::err::Error;
use crate::models::*;
use crate::data;

pub trait ReadOps
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>;

    fn get_num_objects(&self) -> Result<u64, Error>;
    fn get_num_objects_with_attachments(&self) -> Result<u64, Error>;

    fn get_object_by_id(&self, obj_id: String) -> Result<Option<Object>, Error>;
    fn get_objects_by_activity_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_by_modified_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_by_attachment_size_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;

    fn get_attachment_metadata(&self, obj_id: &String) -> Result<Option<AttachmentMetadata>, Error>;
    fn get_attachment_data(&self, obj_id: &String) -> Result<Option<Vec<u8>>, Error>;
}

pub trait WriteOps: ReadOps
{
    fn set_properties(&self, properties: &HashMap<String, String>) -> Result<(), Error>;
    fn add_object(&self, obj_type: data::ObjectType, created_time: Option<data::Date>, activity_time: Option<data::Date>, title: Option<String>, notes: Option<String>, rating: Option<data::Rating>, censor: data::Censor, location: Option<data::Location>) -> Result<Object, Error>;
    fn add_attachment(&self, obj_id: &String, filename: String, created: data::Date, modified: data::Date, mime: String, orientation: Option<data::Orientation>, dimensions: Option<data::Dimensions>, duration: Option<data::Duration>, bytes: Vec<u8>) -> Result<(), Error>;
}
