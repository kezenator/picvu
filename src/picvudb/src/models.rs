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
#[derive(Insertable)]
#[table_name="objects"]
pub struct Object
{
    pub id: String,
    pub created_timestamp: i64,
    pub created_offset: i32,
    pub modified_timestamp: i64,
    pub modified_offset: i32,
    pub activity_timestamp: i64,
    pub activity_offset: i32,
    pub obj_type: String,
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
    pub obj_id: String,
    pub filename: String,
    pub created_timestamp: i64,
    pub created_offset: i32,
    pub modified_timestamp: i64,
    pub modified_offset: i32,
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
    pub obj_id: String,
    pub offset: i64,
    pub bytes: Vec<u8>,
}
