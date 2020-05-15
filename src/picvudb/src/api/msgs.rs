use std::collections::HashMap;
use serde::Serialize;

use crate::Error;
use crate::models;
use crate::api::ApiMessage;
use crate::store::WriteOps;
use crate::api::data::*;

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
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct GetAllObjectsRequest
{
}

impl ApiMessage for GetAllObjectsRequest
{
    type Response = GetAllObjectsResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let to_api_object = |obj: models::Object| { Object {
            id: obj.id,
            added: Date{ timestamp: obj.added_timestamp, timestring: obj.added_timestring, },
            changed: Date{ timestamp: obj.changed_timestamp, timestring: obj.changed_timestring, },
            label: obj.label,
        }};

        let objects = ops.get_all_objects()?
            .drain(..)
            .map(|obj| to_api_object(obj))
            .collect();

        Ok(GetAllObjectsResponse{ objects })
    }
}

#[derive(Debug, Serialize)]
pub struct GetAllObjectsResponse
{
    pub objects: Vec<Object>,
}

#[derive(Debug)]
pub struct AddObjectRequest
{
    pub label: String,
}

impl ApiMessage for AddObjectRequest
{
    type Response = AddObjectResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let to_api_object = |obj: models::Object| { Object {
            id: obj.id,
            added: Date{ timestamp: obj.added_timestamp, timestring: obj.added_timestring, },
            changed: Date{ timestamp: obj.changed_timestamp, timestring: obj.changed_timestring, },
            label: obj.label,
        }};

        let object = to_api_object(ops.add_object(&self.label)?);

        Ok(AddObjectResponse{ object })
    }
}

#[derive(Debug, Serialize)]
pub struct AddObjectResponse
{
    pub object: Object,
}