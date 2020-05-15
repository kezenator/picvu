use diesel::SqliteConnection;
use std::collections::HashMap;

use crate::err::Error;
use crate::store::ops::*;
use crate::queries;
use crate::models;

use diesel::RunQueryDsl;

pub struct Transaction<'a>
{
    pub connection: &'a SqliteConnection,
}

impl<'a> ReadOps for Transaction<'a>
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>
    {
        let props = queries::properties::all()
            .load::<models::DbProperty>(self.connection)?
            .drain(..)
            .map(|prop| { (prop.name, prop.value) })
            .collect();

        Ok(props)
    }
}

impl<'a> WriteOps for Transaction<'a>
{
}
