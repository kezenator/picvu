use diesel::{RunQueryDsl, SqliteConnection};
use std::collections::HashMap;

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
}

impl<'a> WriteOps for Transaction<'a>
{
    fn add_object(&self, label: &String) -> Result<Object, Error>
    {
        let added = data::Date::now();
        let changed = added.clone();
        let id = format!("{}", uuid::Uuid::new_v4());

        let model_object = Object
        {
            id: id.clone(),
            added_timestamp: added.timestamp,
            added_timestring: added.timestring.clone(),
            changed_timestamp: changed.timestamp,
            changed_timestring: changed.timestring.clone(),
            label: label.clone(),
        };

        diesel::insert_into(schema::objects::table)
            .values(&model_object)
            .execute(self.connection)?;

        Ok(model_object)
}
}
