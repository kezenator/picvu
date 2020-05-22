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

        let mut data = attachments_data
            .filter(id.eq(obj_id))
            .order_by(offset.asc())
            .load::<AttachmentData>(self.connection)?;

        if data.is_empty()
        {
            return Ok(None);
        }

        let mut size: usize = 0;
        for d in data.iter()
        {
            if (size as i64) != d.offset
            {
                return Err(Error::DatabaseConsistencyError{
                    msg: format!("Object {} has invalid attachment block offsets", obj_id),
                });
            }

            let new_size = size + d.bytes.len();
            if new_size < size
            {
                return Err(Error::DatabaseConsistencyError{
                    msg: format!("Object {} has an attachment that is too large to fit in memory", obj_id),
                });
            }

            size = new_size;
        }

        let mut collected_bytes = Vec::new();
        collected_bytes.reserve(size);

        for mut data in data.drain(..)
        {
            collected_bytes.append(&mut data.bytes);
        }

        Ok(Some(collected_bytes))
    }
}

impl<'a> WriteOps for Transaction<'a>
{
    fn add_object(&self, title: Option<String>, obj_type: data::ObjectType) -> Result<Object, Error>
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
            obj_type: obj_type.to_db_string(),
            title: title,
        };

        diesel::insert_into(schema::objects::table)
            .values(&model_object)
            .execute(self.connection)?;

        Ok(model_object)
    }

    fn add_attachment(&self, obj_id: &String, filename: String, created: data::Date, modified: data::Date, mime: String, bytes: Vec<u8>) -> Result<(), Error>
    {
        if bytes.is_empty()
        {
            return Err(Error::DatabaseConsistencyError{
                msg: "Cannot insert zero length attacments".to_owned(),
            });
        }

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

        diesel::insert_into(schema::attachments_metadata::table)
            .values(&model_metadata)
            .execute(self.connection)?;

        let mut offset = 0;
        while offset < bytes.len()
        {
            let remaining = bytes.len() - offset;
            let this_time = std::cmp::min(remaining, 512 * 1024);

            let model_data = AttachmentData
            {
                id: obj_id.clone(),
                offset: offset as i64,
                bytes: bytes[offset..(offset + this_time)].to_vec(),
            };
    
            diesel::insert_into(schema::attachments_data::table)
                .values(&model_data)
                .execute(self.connection)?;

            offset += this_time;
        }

        Ok(())
    }
}
