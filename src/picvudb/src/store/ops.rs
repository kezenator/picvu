use std::collections::HashMap;

use crate::err::Error;
use crate::models::*;

pub trait ReadOps
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>;
    fn get_all_objects(&self) -> Result<Vec<Object>, Error>;
}

pub trait WriteOps: ReadOps
{
    fn add_object(&self, label: &String) -> Result<Object, Error>;
}
