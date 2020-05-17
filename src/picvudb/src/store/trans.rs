use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

use crate::err::Error;
use crate::store::ops::*;
use crate::models::*;
use crate::schema;
use crate::api::data;

pub struct Transaction<'a>
{
    pub connection: &'a SqliteConnection,
}

impl<'a> ReadOps for Transaction<'a>
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>
    {
        let props = schema::db_properties::table
            .load::<DbProperty>(self.connection)?
            .drain(..)
            .map(|prop| { (prop.name, prop.value) })
            .collect();

        Ok(props)
    }

    fn get_all_objects(&self) -> Result<Vec<Object>, Error>
    {
        let objects = schema::objects::table
            .load::<Object>(self.connection)?;

        Ok(objects)
    }

    fn get_attachment_metadata(&self, obj_id: &String) -> Result<Option<AttachmentMetadata>, Error>
    {
        use schema::attachments_metadata::dsl::*;

        let metadata = attachments_metadata
            .filter(id.eq(obj_id))
            .load::<AttachmentMetadata>(self.connection)?
            .into_iter()
            .nth(0);

        Ok(metadata)
    }

    fn get_attachment_data(&self, obj_id: &String) -> Result<Option<Vec<u8>>, Error>
    {
        use schema::attachments_data::dsl::*;

        let data = attachments_data
            .filter(id.eq(obj_id))
            .load::<AttachmentData>(self.connection)?
            .into_iter()
            .nth(0);

        match data
        {
            Some(data) => Ok(Some(data.bytes)),
            None => Ok(None),
        }
    }
}

impl<'a> WriteOps for Transaction<'a>
{
    fn add_object(&self, title: Option<String>) -> Result<Object, Error>
    {
        let added = data::Date::now();
        let changed = added.clone();
        let id = format!("{}", uuid::Uuid::new_v4());

        let model_object = Object
        {
            id: id.clone(),
            added_timestamp: added.to_db_timestamp(),
            added_timestring: added.to_db_timestring().clone(),
            changed_timestamp: changed.to_db_timestamp(),
            changed_timestring: changed.to_db_timestring().clone(),
            title: title,
        };

        diesel::insert_into(schema::objects::table)
            .values(&model_object)
            .execute(self.connection)?;

        Ok(model_object)
    }

    fn add_attachment(&self, obj_id: &String, filename: String, created: data::Date, modified: data::Date, mime: String, bytes: Vec<u8>) -> Result<(), Error>
    {
        let hash =
        {
            let mut hasher = Sha256::new();
            hasher.input(&bytes);
            format!("{}-sha256", base16::encode_lower(&hasher.result()))
        };

        let model_metadata = AttachmentMetadata
        {
            id: obj_id.clone(),
            filename: filename,
            created: created.to_db_timestamp(),
            modified: modified.to_db_timestamp(),
            mime: mime,
            size: bytes.len() as i64,
            hash: hash,
        };

        let model_data = AttachmentData
        {
            id: obj_id.clone(),
            bytes: bytes,
        };

        diesel::insert_into(schema::attachments_metadata::table)
            .values(&model_metadata)
            .execute(self.connection)?;

        diesel::insert_into(schema::attachments_data::table)
            .values(&model_data)
            .execute(self.connection)?;

        Ok(())
    }
}
