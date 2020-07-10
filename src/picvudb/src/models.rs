use super::schema::*;

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="db_properties"]
pub struct DbProperty
{
    pub name: String,
    pub value: String,
}

#[derive(Clone)]
#[derive(Queryable)]
pub struct Object
{
    pub id: i64,
    pub created_timestamp: i64,
    pub created_offset: Option<i32>,
    pub modified_timestamp: i64,
    pub modified_offset: Option<i32>,
    pub activity_timestamp: i64,
    pub activity_offset: Option<i32>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub censor: i32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub tag_set: Option<String>,
    pub ext_ref_type: Option<String>,
    pub ext_ref_id: Option<String>,
}

#[derive(Insertable)]
#[table_name="objects"]
pub struct InsertableObject
{
    pub created_timestamp: i64,
    pub created_offset: Option<i32>,
    pub modified_timestamp: i64,
    pub modified_offset: Option<i32>,
    pub activity_timestamp: i64,
    pub activity_offset: Option<i32>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub censor: i32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub tag_set: Option<String>,
    pub ext_ref_type: Option<String>,
    pub ext_ref_id: Option<String>,
}

#[derive(QueryableByName)]
pub struct NewId
{
    #[sql_type = "diesel::sql_types::BigInt"]
    pub new_id: i64
}

#[derive(Identifiable)]
#[table_name="objects"]
pub struct UpdateObjectId
{
    pub id: i64,
}

#[derive(AsChangeset)]
#[table_name="objects"]
#[changeset_options(treat_none_as_null="true")]
pub struct UpdateObjectChangeset
{
    pub modified_timestamp: i64,
    pub modified_offset: Option<i32>,
    pub activity_timestamp: i64,
    pub activity_offset: Option<i32>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub censor: i32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="attachments_metadata"]
pub struct AttachmentMetadata
{
    pub obj_id: i64,
    pub filename: String,
    pub created_timestamp: i64,
    pub created_offset: Option<i32>,
    pub modified_timestamp: i64,
    pub modified_offset: Option<i32>,
    pub mime: String,
    pub size: i64,
    pub orientation: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub hash: String,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="attachments_data"]
pub struct AttachmentData
{
    pub obj_id: i64,
    pub offset: i64,
    pub bytes: Vec<u8>,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="objects_fts"]
pub struct ObjectsFts
{
    pub id: i64,
    pub title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="objects_location"]
pub struct ObjectsLocation
{
    pub id: i64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_long: f64,
    pub max_long: f64,
}

#[derive(Insertable)]
#[table_name="tags"]
pub struct InsertableTag
{
    pub tag_name: String,
    pub tag_kind: i32,
    pub tag_rating: Option<i32>,
    pub tag_censor: i32,
}

#[derive(Queryable)]
pub struct Tag
{
    pub tag_id: i64,
    pub tag_name: String,
    pub tag_kind: i32,
    pub tag_rating: Option<i32>,
    pub tag_censor: i32,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="object_tags"]
pub struct ObjectTags
{
    pub obj_id: i64,
    pub tag_id: i64,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="tags_fts"]
pub struct TagsFts
{
    pub tag_id: i64,
    pub tag_name: String,
}

#[derive(Identifiable)]
#[table_name="tags"]
#[primary_key(tag_id)]
pub struct UpdateTagId
{
    pub tag_id: i64,
}

#[derive(AsChangeset)]
#[table_name="tags"]
#[changeset_options(treat_none_as_null="true")]
pub struct UpdateTagChangeset
{
    pub tag_name: String,
    pub tag_rating: Option<i32>,
    pub tag_censor: i32,
    pub tag_kind: i32,
}
