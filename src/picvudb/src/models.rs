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
    pub created_timestring: String,
    pub modified_timestamp: i64,
    pub modified_timestring: String,
    pub activity_timestamp: i64,
    pub activity_timestring: String,
    pub obj_type: String,
    pub title: Option<String>,
    pub notes: Option<String>,
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
    pub created: i64,
    pub modified: i64,
    pub mime: String,
    pub size: i64,
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
