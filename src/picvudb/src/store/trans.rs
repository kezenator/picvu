use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use num_traits::cast::ToPrimitive;

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

    fn get_num_objects(&self) -> Result<u64, Error>
    {
        use schema::objects::dsl::*;
        use diesel::dsl::count_star;

        let num: u64 = objects
            .select(count_star())
            .first::<i64>(self.connection)?
            .to_u64()
            .ok_or(Error::DatabaseConsistencyError{ msg: "More than 2^64 objects in database".to_owned() })?;

        Ok(num)
    }

    fn get_num_objects_with_attachments(&self) -> Result<u64, Error>
    {
        use schema::attachments_metadata::dsl::*;
        use diesel::dsl::count_star;

        let num: u64 = attachments_metadata
            .select(count_star())
            .first::<i64>(self.connection)?
            .to_u64()
            .ok_or(Error::DatabaseConsistencyError{ msg: "More than 2^64 attachments in database".to_owned() })?;

        Ok(num)
    }

    fn get_num_objects_near_location(&self, latitude: f64, longitude: f64, radius_meters: f64) -> Result<u64, Error>
    {
        // There are approximately 100 km per degree (lat or long) at
        // the equator
        //
        // TODO - use a real library to calculate the range

        let radius_degrees = radius_meters / 100000.0;

        let q_min_lat = latitude - radius_degrees;
        let q_max_lat = latitude + radius_degrees;
        let q_min_long = longitude - radius_degrees;
        let q_max_long = longitude + radius_degrees;

        use schema::objects_location::dsl::*;
        use diesel::dsl::count_star;

        let num: u64 = objects_location
            .select(count_star())
            .filter(min_lat.ge(q_min_lat)
                .and(max_lat.le(q_max_lat))
                .and(min_long.ge(q_min_long))
                .and(max_long.le(q_max_long)))
            .first::<i64>(self.connection)?
            .to_u64()
            .ok_or(Error::DatabaseConsistencyError{ msg: "More than 2^64 objects in database".to_owned() })?;

        Ok(num)
    }

    fn get_object_by_id(&self, obj_id: i64) -> Result<Option<Object>, Error>
    {
        use schema::objects::dsl::*;

        let object = objects
            .filter(id.eq(obj_id))
            .first::<Object>(self.connection)
            .optional()?;

        Ok(object)
    }

    fn get_objects_by_activity_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        use schema::objects::dsl::*;

        let results = objects
            .order_by(activity_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;

        Ok(results)
    }

    fn get_objects_by_modified_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        use schema::objects::dsl::*;

        let results = objects
            .order_by(modified_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;

        Ok(results)
    }

    fn get_objects_by_attachment_size_desc(&self, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        use schema::objects::dsl::*;

        let results = objects
            .inner_join(schema::attachments_metadata::table)
            .select(schema::objects::all_columns)
            .order_by(schema::attachments_metadata::size.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;

        Ok(results)
    }

    fn get_objects_near_location_by_activity_desc(&self, latitude: f64, longitude: f64, radius_meters: f64, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        // There are approximately 100 km per degree (lat or long) at
        // the equator
        //
        // TODO - use a real library to calculate the range

        let radius_degrees = radius_meters / 100000.0;

        let q_min_lat = latitude - radius_degrees;
        let q_max_lat = latitude + radius_degrees;
        let q_min_long = longitude - radius_degrees;
        let q_max_long = longitude + radius_degrees;

        //use schema::objects::dsl::*;
        use schema::objects_location::dsl::*;

        let results = schema::objects::table
            .inner_join(schema::objects_location::table)
            .filter(min_lat.ge(q_min_lat)
                .and(max_lat.le(q_max_lat))
                .and(min_long.ge(q_min_long))
                .and(max_long.le(q_max_long)))
            .order_by(schema::objects::activity_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<(Object, ObjectsLocation)>(self.connection)?
            .drain(..)
            .map(|(o, _l)| o)
            .collect();

        Ok(results)
    }

    fn get_attachment_metadata(&self, q_obj_id: i64) -> Result<Option<AttachmentMetadata>, Error>
    {
        use schema::attachments_metadata::dsl::*;

        let metadata = attachments_metadata
            .filter(obj_id.eq(q_obj_id))
            .load::<AttachmentMetadata>(self.connection)?
            .into_iter()
            .nth(0);

        Ok(metadata)
    }

    fn get_attachment_data(&self, q_obj_id: i64) -> Result<Option<Vec<u8>>, Error>
    {
        use schema::attachments_data::dsl::*;

        let mut data = attachments_data
            .filter(obj_id.eq(q_obj_id))
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
                    msg: format!("Object {} has invalid attachment block offsets", q_obj_id),
                });
            }

            let new_size = size + d.bytes.len();
            if new_size < size
            {
                return Err(Error::DatabaseConsistencyError{
                    msg: format!("Object {} has an attachment that is too large to fit in memory", q_obj_id),
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
    fn set_properties(&self, properties: &HashMap<String, String>) -> Result<(), Error>
    {
        let existing_properties = self.get_properties()?;

        for (new_name, new_value) in properties
        {
            if existing_properties.contains_key(new_name)
            {
                use schema::db_properties::dsl::*;

                let target = db_properties.filter(name.eq(new_name));

                diesel::update(target)
                    .set(value.eq(new_value))
                    .execute(self.connection)?;
            }
            else // insert new property
            {
                diesel::insert_into(schema::db_properties::table)
                    .values(&DbProperty{name: new_name.clone(), value: new_value.clone()})
                    .execute(self.connection)?;
            }
        }

        Ok(())
    }

    fn add_object(&self, created_time: Option<data::Date>, activity_time: Option<data::Date>, title: Option<String>, notes: Option<String>, rating: Option<data::Rating>, censor: data::Censor, location: Option<data::Location>) -> Result<Object, Error>
    {
        let modified_time = data::Date::now();
        let created_time = created_time.unwrap_or(modified_time.clone());
        let activity_time = activity_time.unwrap_or(created_time.clone());

        let latitude = location.clone().map(|l| l.latitude);
        let longitude = location.clone().map(|l| l.longitude);

        let insertable_object = InsertableObject
        {
            created_timestamp: created_time.to_db_timestamp(),
            created_offset: created_time.to_db_offset(),
            modified_timestamp: modified_time.to_db_timestamp(),
            modified_offset: modified_time.to_db_offset(),
            activity_timestamp: activity_time.to_db_timestamp(),
            activity_offset: activity_time.to_db_offset(),
            title: title.clone(),
            notes: notes.clone(),
            rating: rating.clone().map(|r| { r.to_db_field() }),
            censor: censor.to_db_field(),
            latitude: latitude,
            longitude: longitude,
        };

        diesel::insert_into(schema::objects::table)
            .values(&insertable_object)
            .execute(self.connection)?;

        let new_id: NewId = diesel::sql_query("SELECT last_insert_rowid() as 'new_id'")
            .get_result(self.connection)?;

        // Now add data to the extra search indexes

        if title.is_some() || notes.is_some()
        {
            let fts_insert_value = ObjectsFts
            {
                id: new_id.new_id,
                title: title.clone(),
                notes: notes.clone(),
            };

            diesel::insert_into(schema::objects_fts::table)
                .values(vec![fts_insert_value])
                .execute(self.connection)?;
        }

        if let (Some(lat), Some(long)) = (latitude, longitude)
        {
            let location_insert_value = ObjectsLocation
            {
                id: new_id.new_id,
                min_lat: lat,
                max_lat: lat,
                min_long: long,
                max_long: long,
            };

            diesel::insert_into(schema::objects_location::table)
                .values(vec![location_insert_value])
                .execute(self.connection)?;
        }

        // Return the created object

        let model_object = Object
        {
            id: new_id.new_id,
            created_timestamp: created_time.to_db_timestamp(),
            created_offset: created_time.to_db_offset(),
            modified_timestamp: modified_time.to_db_timestamp(),
            modified_offset: modified_time.to_db_offset(),
            activity_timestamp: activity_time.to_db_timestamp(),
            activity_offset: activity_time.to_db_offset(),
            title: title,
            notes: notes,
            rating: rating.map(|r| { r.to_db_field() }),
            censor: censor.to_db_field(),
            latitude: latitude,
            longitude: longitude,
        };

        Ok(model_object)
    }

    fn add_attachment(&self, obj_id: i64, filename: String, created: data::Date, modified: data::Date, mime: String, orientation: Option<data::Orientation>, dimensions: Option<data::Dimensions>, duration: Option<data::Duration>, bytes: Vec<u8>) -> Result<(), Error>
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
            obj_id: obj_id,
            filename: filename,
            created_timestamp: created.to_db_timestamp(),
            created_offset: created.to_db_offset(),
            modified_timestamp: modified.to_db_timestamp(),
            modified_offset: modified.to_db_offset(),
            mime: mime,
            size: bytes.len() as i64,
            orientation: orientation.map(|o| o.to_db_field()),
            width: dimensions.clone().map(|d| d.to_db_field_width()),
            height: dimensions.clone().map(|d| d.to_db_field_height()),
            duration: duration.map(|d| d.to_db_field()),
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
                obj_id: obj_id.clone(),
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

    fn update_object(&self, obj_id: i64, activity_time: data::Date, title: Option<String>, notes: Option<String>, rating: Option<data::Rating>, censor: data::Censor, location: Option<data::Location>) -> Result<(), Error>
    {
        let object = UpdateObjectId
        {
            id: obj_id,
        };

        let changeset = UpdateObjectChangeset
        {
            activity_timestamp: activity_time.to_db_timestamp(),
            activity_offset: activity_time.to_db_offset(),
            title: title.clone(),
            notes: notes.clone(),
            rating: rating.map(|r| r.to_db_field()),
            censor: censor.to_db_field(),
            latitude: location.clone().map(|l| l.latitude),
            longitude: location.clone().map(|l| l.longitude),
        };

        diesel::update(&object).set(changeset).execute(self.connection)?;

        // Update the associated indexes

        if title.is_some() || notes.is_some()
        {
            let fts_insert_value = ObjectsFts
            {
                id: obj_id,
                title: title,
                notes: notes,
            };

            diesel::replace_into(schema::objects_fts::table)
                .values(vec![fts_insert_value])
                .execute(self.connection)?;
        }
        else
        {
            use schema::objects_fts::dsl::*;

            diesel::delete(objects_fts.filter(id.eq(obj_id)))
                .execute(self.connection)?;
        }

        if let Some(loc) = location
        {
            let location_insert_value = ObjectsLocation
            {
                id: obj_id,
                min_lat: loc.latitude,
                max_lat: loc.latitude,
                min_long: loc.longitude,
                max_long: loc.longitude,
            };

            diesel::replace_into(schema::objects_location::table)
                .values(vec![location_insert_value])
                .execute(self.connection)?;
        }
        else
        {
            use schema::objects_location::dsl::*;

            diesel::delete(objects_location.filter(id.eq(obj_id)))
                .execute(self.connection)?;
        }

        Ok(())
    }
}
