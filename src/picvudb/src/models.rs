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
    pub label: String,
}
