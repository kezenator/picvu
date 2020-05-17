use super::schema::*;

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="db_properties"]
pub struct DbProperty
{
    pub name: String,
    pub value: String,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="objects"]
pub struct Object
{
    pub id: String,
    pub added_timestamp: i64,
    pub added_timestring: String,
    pub changed_timestamp: i64,
    pub changed_timestring: String,
    pub title: Option<String>,
}

#[derive(Queryable)]
#[derive(Insertable)]
#[table_name="attachments_metadata"]
pub struct AttachmentMetadata
{
    pub id: String,
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
    pub id: String,
    pub bytes: Vec<u8>,
}
