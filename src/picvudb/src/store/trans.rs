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
use crate::store::extensions::*;

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

    fn get_num_objects_for_text_search(&self, search: &data::get::SearchString) -> Result<u64, Error>
    {
        let fts5_search = search.to_fts5_query();
        let literal_text = search.to_literal_string();

        let num = schema::objects::table
            .select(diesel::dsl::count_star())
            .filter(
                schema::objects::id.eq_any(
                    schema::objects_fts::table
                    .select(schema::objects_fts::id)
                    .filter(schema::objects_fts::dsl::title.fts_match(&fts5_search)
                        .or(schema::objects_fts::dsl::notes.fts_match(&fts5_search))))
                .or(schema::objects::id.eq_any(
                    schema::attachments_metadata::table
                    .select(schema::attachments_metadata::obj_id)
                    .filter(schema::attachments_metadata::dsl::filename.eq(&literal_text)))))
            .first::<i64>(self.connection)?
            .to_u64()
            .ok_or(Error::DatabaseConsistencyError{ msg: "More than 2^64 objects in database".to_owned() })?;

        Ok(num)
    }

    fn get_num_objects_with_tag(&self, tag: i64) -> Result<u64, Error>
    {
        use schema::object_tags::dsl::*;
        use diesel::dsl::count_star;

        let num: u64 = object_tags
            .select(count_star())
            .filter(tag_id.eq(tag))
            .first::<i64>(self.connection)?
            .to_u64()
            .ok_or(Error::DatabaseConsistencyError{ msg: "More than 2^64 objects in database".to_owned() })?;

        Ok(num)
    }

    fn get_num_objects_in_activity_date_range(&self, date_range: &data::DateRange) -> Result<u64, Error>
    {
        use diesel::dsl::count_star;

        let start_ts_utc = date_range.start.first_timestamp_utc_false_positive().unwrap().timestamp();
        let start_ts_local = date_range.start.first_timestamp_after_local_adjust().unwrap().timestamp();
        let end_ts_utc = date_range.end.last_timestamp_utc_false_positive().unwrap().timestamp();
        let end_ts_local = date_range.end.last_timestamp_after_local_adjust().unwrap().timestamp();

        let num = schema::objects::table
            .select(count_star())
            .filter(
                schema::objects::activity_timestamp.ge(start_ts_utc)
                .and(schema::objects::activity_timestamp.le(end_ts_utc))
                .and((schema::objects::activity_timestamp + coalesce(schema::objects::activity_offset, 36000)).ge(start_ts_local))
                .and((schema::objects::activity_timestamp + coalesce(schema::objects::activity_offset, 36000)).le(end_ts_local)))
            .order_by(schema::objects::activity_timestamp.desc())
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

    fn get_objects_for_text_search(&self, search: &data::get::SearchString, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        let fts5_search = search.to_fts5_query();
        let literal_text = search.to_literal_string();

        let results = schema::objects::table
            .filter(
                schema::objects::id.eq_any(
                    schema::objects_fts::table
                    .select(schema::objects_fts::id)
                    .filter(schema::objects_fts::dsl::title.fts_match(&fts5_search)
                        .or(schema::objects_fts::dsl::notes.fts_match(&fts5_search))))
                .or(schema::objects::id.eq_any(
                    schema::attachments_metadata::table
                    .select(schema::attachments_metadata::obj_id)
                    .filter(schema::attachments_metadata::dsl::filename.eq(&literal_text)))))
            .order_by(schema::objects::activity_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;
    
        Ok(results)
    }

    fn get_objects_with_tag_by_activity_desc(&self, tag_id: i64, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        let results = schema::objects::table
            .filter(schema::objects::id.eq_any(
                    schema::object_tags::table.select(schema::object_tags::obj_id).filter(schema::object_tags::tag_id.eq(tag_id))))
            .order_by(schema::objects::activity_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;
    
        Ok(results)
    }

    fn get_objects_in_activity_date_range(&self, date_range: &data::DateRange, offset: u64, page_size: u64) -> Result<Vec<Object>, Error>
    {
        let start_ts_utc = date_range.start.first_timestamp_utc_false_positive().unwrap().timestamp();
        let start_ts_local = date_range.start.first_timestamp_after_local_adjust().unwrap().timestamp();
        let end_ts_utc = date_range.end.last_timestamp_utc_false_positive().unwrap().timestamp();
        let end_ts_local = date_range.end.last_timestamp_after_local_adjust().unwrap().timestamp();

        let results = schema::objects::table
            .filter(
                schema::objects::activity_timestamp.ge(start_ts_utc)
                .and(schema::objects::activity_timestamp.le(end_ts_utc))
                .and((schema::objects::activity_timestamp + coalesce(schema::objects::activity_offset, 36000)).ge(start_ts_local))
                .and((schema::objects::activity_timestamp + coalesce(schema::objects::activity_offset, 36000)).le(end_ts_local)))
            .order_by(schema::objects::activity_timestamp.desc())
            .offset(offset as i64)
            .limit(page_size as i64)
            .load::<Object>(self.connection)?;
    
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

    fn get_tag(&self, tag_id: i64) -> Result<Tag, Error>
    {
        match schema::tags::table
            .filter(schema::tags::tag_id.eq(tag_id))
            .get_result(self.connection)
            .optional()?
        {
            Some(tag) =>
            {
                Ok(tag)
            },
            None =>
            {
                Err(Error::DatabaseConsistencyError{ msg: format!("Tag ID {} has no stored data", tag_id) })
            },
        }
    }

    fn get_tags_for_text_search(&self, search: &data::get::SearchString) -> Result<Vec<Tag>, Error>
    {
        let fts5_search = search.to_fts5_query();

        let results = schema::tags::table
            .filter(
                schema::tags::tag_id.eq_any(
                    schema::tags_fts::table
                    .select(schema::tags_fts::tag_id)
                    .filter(schema::tags_fts::dsl::tag_name.fts_match(&fts5_search))))
            .order_by(schema::tags::tag_name.asc())
            .load::<Tag>(self.connection)?;

        Ok(results)
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

    fn add_object(&self, created_time: Option<data::Date>, modified_time: Option<data::Date>, activity_time: Option<data::Date>, title: Option<data::TitleMarkdown>, notes: Option<data::NotesMarkdown>, rating: data::Rating, censor: data::Censor, location: Option<data::Location>, tag_set: data::TagSet, ext_ref: Option<data::ExternalReference>) -> Result<data::ObjectId, Error>
    {
        let modified_time = modified_time.unwrap_or(data::Date::now());
        let created_time = created_time.unwrap_or(modified_time.clone());
        let activity_time = activity_time.unwrap_or(created_time.clone());

        let location_source = location.clone().map(|l| l.source.to_db_field());
        let latitude = location.clone().map(|l| l.latitude);
        let longitude = location.clone().map(|l| l.longitude);
        let altitude = location.clone().map(|l| l.altitude).flatten();

        let insertable_object = InsertableObject
        {
            created_timestamp: created_time.to_db_timestamp(),
            created_offset: created_time.to_db_offset(),
            modified_timestamp: modified_time.to_db_timestamp(),
            modified_offset: modified_time.to_db_offset(),
            activity_timestamp: activity_time.to_db_timestamp(),
            activity_offset: activity_time.to_db_offset(),
            title: title.clone().map(|m| m.get_markdown()),
            notes: notes.clone().map(|m| m.get_markdown()),
            rating: rating.to_db_field(),
            censor: censor.to_db_field(),
            location_source: location_source,
            latitude: latitude,
            longitude: longitude,
            altitude: altitude,
            tag_set: tag_set.to_db_field(),
            ext_ref_type: ext_ref.clone().map(|e| e.to_db_field_type()),
            ext_ref_id: ext_ref.clone().map(|e| e.to_db_field_id()),
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
                title: title.map(|m| m.get_search_text()),
                notes: notes.map(|m| m.get_search_text()),
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

        // Return the created object ID

        Ok(data::ObjectId::from_db_field(new_id.new_id))
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
            hasher.update(&bytes);
            format!("{}-sha256", base16::encode_lower(&hasher.finalize()))
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

    fn update_object(&self, obj_id: i64, activity_time: data::Date, title: Option<data::TitleMarkdown>, notes: Option<data::NotesMarkdown>, rating: data::Rating, censor: data::Censor, location: Option<data::Location>) -> Result<(), Error>
    {
        let object = UpdateObjectId
        {
            id: obj_id,
        };

        let modified = data::Date::now();

        let changeset = UpdateObjectChangeset
        {
            modified_timestamp: modified.to_db_timestamp(),
            modified_offset: modified.to_db_offset(),
            activity_timestamp: activity_time.to_db_timestamp(),
            activity_offset: activity_time.to_db_offset(),
            title: title.clone().map(|m| m.get_markdown()),
            notes: notes.clone().map(|m| m.get_markdown()),
            rating: rating.to_db_field(),
            censor: censor.to_db_field(),
            location_source: location.clone().map(|l| l.source.to_db_field()),
            latitude: location.clone().map(|l| l.latitude),
            longitude: location.clone().map(|l| l.longitude),
            altitude: location.clone().map(|l| l.altitude).flatten(),
        };

        diesel::update(&object).set(changeset).execute(self.connection)?;

        // Update the associated indexes
        // Always delete, then re-add if there is
        // still data. The SQLite VIRTUAL TABLE implementations
        // don't seem to handle diesel "update_into" very well.

        // Full text search

        {
            use schema::objects_fts::dsl::*;

            diesel::delete(objects_fts.filter(id.eq(obj_id)))
                .execute(self.connection)?;
        }

        if title.is_some() || notes.is_some()
        {
            let fts_insert_value = ObjectsFts
            {
                id: obj_id,
                title: title.map(|m| m.get_search_text()),
                notes: notes.map(|m| m.get_search_text()),
            };

            diesel::insert_into(schema::objects_fts::table)
                .values(vec![fts_insert_value])
                .execute(self.connection)?;
        }

        // Location

        {
            use schema::objects_location::dsl::*;

            diesel::delete(objects_location.filter(id.eq(obj_id)))
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

            diesel::insert_into(schema::objects_location::table)
                .values(vec![location_insert_value])
                .execute(self.connection)?;
        }

        Ok(())
    }

    fn update_object_tagset(&self, obj_id: i64, tag_set: data::TagSet) -> Result<(), Error>
    {
        let object = UpdateObjectId
        {
            id: obj_id,
        };

        let modified = data::Date::now();

        let changeset = UpdateObjectTagsetChangeset
        {
            modified_timestamp: modified.to_db_timestamp(),
            modified_offset: modified.to_db_offset(),
            tag_set: tag_set.to_db_field(),
        };

        diesel::update(&object).set(changeset).execute(self.connection)?;

        Ok(())
    }

    fn update_tag(&self, tag_id: data::TagId, name: String, rating: data::Rating, censor: data::Censor, kind: data::TagKind) -> Result<(), Error>
    {
        // First, check there's no other tag with this name

        match schema::tags::table.filter(schema::tags::tag_name.eq(name.clone())).get_result::<Tag>(self.connection).optional()?
        {
            Some(tag) =>
            {
                if tag.tag_id != tag_id.to_db_field()
                {
                    return Err(Error::DatabaseConsistencyError{ msg: format!("There is already another tag with the name {:?}", name) });
                }
            },
            None =>
            {
                // OK!
            },
        }

        // Update the tag

        let tag = UpdateTagId
        {
            tag_id: tag_id.to_db_field(),
        };

        let changeset = UpdateTagChangeset
        {
            tag_name: name.clone(),
            tag_rating: rating.to_db_field(),
            tag_censor: censor.to_db_field(),
            tag_kind: kind.to_db_field(),
        };

        diesel::update(&tag).set(changeset).execute(self.connection)?;

        // Update the full-text search

        {
            diesel::delete(schema::tags_fts::table.filter(schema::tags_fts::dsl::tag_id.eq(tag_id.to_db_field())))
                .execute(self.connection)?;

            let fts_insert_value = TagsFts
            {
                tag_id: tag_id.to_db_field(),
                tag_name: name.clone(),
            };

            diesel::insert_into(schema::tags_fts::table)
                .values(vec![fts_insert_value])
                .execute(self.connection)?;
        }

        Ok(())
    }

    fn delete_tag(&self, tag_id: &data::TagId) -> Result<(), Error>
    {
        // First, check there's no other tag with this name

        if schema::object_tags::table
            .filter(schema::object_tags::tag_id.eq(tag_id.to_db_field()))
            .load::<ObjectTags>(self.connection)?
            .len() != 0
        {
            return Err(Error::DatabaseConsistencyError{ msg: format!("Tag {:?} can't be deleted - some objects are still tagged with it", tag_id) });
        }

        // Delete the tag
        // and the Full-Text-Search entry

        diesel::delete(schema::tags::table.filter(schema::tags::dsl::tag_id.eq(tag_id.to_db_field())))
            .execute(self.connection)?;

        diesel::delete(schema::tags_fts::table.filter(schema::tags_fts::dsl::tag_id.eq(tag_id.to_db_field())))
            .execute(self.connection)?;

        Ok(())
    }

    fn find_or_add_tag(&self, name: String, kind: data::TagKind, rating: data::Rating, censor: data::Censor) -> Result<i64, Error>
    {
        match schema::tags::table.filter(schema::tags::tag_name.eq(name.clone())).get_result::<Tag>(self.connection).optional()?
        {
            Some(tag) =>
            {
                Ok(tag.tag_id)
            },
            None =>
            {
                let tag = InsertableTag
                {
                    tag_name: name.clone(),
                    tag_kind: kind.to_db_field(),
                    tag_rating: rating.to_db_field(),
                    tag_censor: censor.to_db_field(),
                };

                diesel::insert_into(schema::tags::table)
                    .values(&tag)
                    .execute(self.connection)?;

                let new_id: NewId = diesel::sql_query("SELECT last_insert_rowid() as 'new_id'")
                    .get_result(self.connection)?;

                let tag_fts = TagsFts
                {
                    tag_id: new_id.new_id,
                    tag_name: name
                };

                diesel::insert_into(schema::tags_fts::table)
                    .values(&tag_fts)
                    .execute(self.connection)?;

                Ok(new_id.new_id)
            },
        }
    }

    fn add_object_tag(&self, obj_id: i64, tag_id: i64) -> Result<(), Error>
    {
        let entry = ObjectTags
        {
            obj_id: obj_id,
            tag_id: tag_id,
        };

        diesel::insert_into(schema::object_tags::table)
            .values(&entry)
            .execute(self.connection)?;

        Ok(())
    }

    fn remove_object_tag(&self, obj_id: i64, tag_id: i64) -> Result<(), Error>
    {
        diesel::delete(
            schema::object_tags::table.filter(
                schema::object_tags::dsl::obj_id.eq(obj_id)
                .and(schema::object_tags::dsl::tag_id.eq(tag_id))))
        .execute(self.connection)?;

        Ok(())
    }
}
