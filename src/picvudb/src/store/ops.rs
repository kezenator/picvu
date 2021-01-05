use std::collections::HashMap;

use crate::err::Error;
use crate::models::*;
use crate::data;

pub trait ReadOps
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>;

    fn get_num_objects(&self) -> Result<u64, Error>;
    fn get_num_objects_with_attachments(&self) -> Result<u64, Error>;
    fn get_num_objects_near_location(&self, latitude: f64, longitude: f64, radius_meters: f64) -> Result<u64, Error>;
    fn get_num_objects_for_text_search(&self, search: &data::get::SearchString) -> Result<u64, Error>;
    fn get_num_objects_with_tag(&self, tag: i64) -> Result<u64, Error>;
    fn get_num_objects_in_activity_date_range(&self, date_range: &data::DateRange) -> Result<u64, Error>;

    fn get_object_by_id(&self, obj_id: i64) -> Result<Option<Object>, Error>;
    fn get_objects_by_activity_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_by_modified_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_by_attachment_size_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_near_location_by_activity_desc(&self, latitude: f64, longitude: f64, radius_meters: f64, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_for_text_search(&self, search: &data::get::SearchString, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_with_tag_by_activity_desc(&self, tag_id: i64, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;
    fn get_objects_in_activity_date_range(&self, date_range: &data::DateRange, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>;

    fn get_attachment_metadata(&self, obj_id: i64) -> Result<Option<AttachmentMetadata>, Error>;
    fn get_attachment_data(&self, obj_id: i64) -> Result<Option<Vec<u8>>, Error>;

    fn get_tag(&self, tag_id: i64) -> Result<Tag, Error>;
    fn get_tags_for_text_search(&self, search: &data::get::SearchString) -> Result<Vec<Tag>, Error>;
}

pub trait WriteOps: ReadOps
{
    fn set_properties(&self, properties: &HashMap<String, String>) -> Result<(), Error>;
    fn add_object(&self, created_time: Option<data::Date>, modified_time: Option<data::Date>, activity_time: Option<data::Date>, title: Option<data::TitleMarkdown>, notes: Option<data::NotesMarkdown>, rating: Option<data::Rating>, censor: data::Censor, location: Option<data::Location>, tag_set: data::TagSet, ext_ref: Option<data::ExternalReference>) -> Result<data::ObjectId, Error>;
    fn add_attachment(&self, obj_id: i64, filename: String, created: data::Date, modified: data::Date, mime: String, orientation: Option<data::Orientation>, dimensions: Option<data::Dimensions>, duration: Option<data::Duration>, bytes: Vec<u8>) -> Result<(), Error>;
    fn update_object(&self, obj_id: i64, activity_time: data::Date, title: Option<data::TitleMarkdown>, notes: Option<data::NotesMarkdown>, rating: Option<data::Rating>, censor: data::Censor, location: Option<data::Location>) -> Result<(), Error>;
    fn update_object_tagset(&self, obj_id: i64, tag_set: data::TagSet) -> Result<(), Error>;
    fn update_tag(&self, tag_id: data::TagId, name: String, rating: Option<data::Rating>, censor: data::Censor, kind: data::TagKind) -> Result<(), Error>;
    fn delete_tag(&self, tag_id: &data::TagId) -> Result<(), Error>;
    fn find_or_add_tag(&self, name: String, kind: data::TagKind, rating: Option<data::Rating>, censor: data::Censor) -> Result<i64, Error>;
    fn add_object_tag(&self, obj_id: i64, tag_id: i64) -> Result<(), Error>;
    fn remove_object_tag(&self, obj_id: i64, tag_id: i64) -> Result<(), Error>;
}
