use std::collections::HashMap;

use crate::err::Error;

pub trait ReadOps
{
    fn get_properties(&self) -> Result<HashMap<String, String>, Error>;
}

pub trait WriteOps: ReadOps
{
}
