use std::collections::HashMap;
use serde::Serialize;

use crate::Error;
use crate::api::ApiMessage;
use crate::store::WriteOps;

#[derive(Debug)]
pub struct GetPropertiesRequest
{
}

impl ApiMessage for GetPropertiesRequest
{
    type Response = GetPropertiesResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        Ok(GetPropertiesResponse{
            properties: ops.get_properties()?
        })
    }
}

#[derive(Debug, Serialize)]
pub struct GetPropertiesResponse
{
    properties: HashMap<String, String>,
}