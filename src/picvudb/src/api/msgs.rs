use std::collections::HashMap;

use crate::Error;
use crate::api::ApiMessage;
use crate::store::WriteOps;
use crate::api::data;

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

#[derive(Debug)]
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
        let mut results = Vec::new();

        let mut from_db = ops.get_all_objects()?;
        results.reserve(from_db.len());

        for object in from_db.drain(..)
        {
            let mut additional = data::get::AdditionalMetadata::None;

            if let Some(attachment) = ops.get_attachment_metadata(&object.id)?
            {
                additional = data::get::AdditionalMetadata::Photo(data::get::PhotoMetadata
                {
                    attachment: data::get::AttachmentMetadata
                    {
                        filename: attachment.filename,
                        created: data::Date::from_db_timestamp(attachment.created),
                        modified: data::Date::from_db_timestamp(attachment.modified),
                        mime: attachment.mime.parse::<mime::Mime>()?,
                        size: attachment.size as u64,
                        hash: attachment.hash,
                    },
                });
            }

            results.push(data::get::ObjectMetadata
            {
                id: data::ObjectId::new(object.id),
                added: data::Date::from_db_fields(object.added_timestamp, object.added_timestring),
                changed: data::Date::from_db_fields(object.changed_timestamp, object.changed_timestring),
                title: object.title,
                additional: additional,
            });
        }

        Ok(GetAllObjectsResponse{ objects: results })
    }
}

#[derive(Debug)]
pub struct GetAllObjectsResponse
{
    pub objects: Vec<data::get::ObjectMetadata>,
}

#[derive(Debug)]
pub struct AddObjectRequest
{
    pub data: data::add::ObjectData,
}

impl ApiMessage for AddObjectRequest
{
    type Response = AddObjectResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let object = ops.add_object(self.data.title.clone())?;

        match &self.data.additional
        {
            data::add::AdditionalData::Photo{attachment} =>
            {
                ops.add_attachment(
                    &object.id,
                    attachment.filename.clone(),
                    attachment.created.clone(),
                    attachment.modified.clone(),
                    attachment.mime.to_string(),
                    attachment.bytes.clone())?;
            },
        };

        Ok(AddObjectResponse{ object_id: data::ObjectId::new(object.id) })
    }
}

#[derive(Debug)]
pub struct AddObjectResponse
{
    pub object_id: data::ObjectId,
}

#[derive(Debug)]
pub struct GetAttachmentDataRequest
{
    pub object_id: data::ObjectId,
    pub specific_hash: Option<String>,
}

impl ApiMessage for GetAttachmentDataRequest
{
    type Response = GetAttachmentDataResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let metadata = ops.get_attachment_metadata(&self.object_id.0)?;
        match metadata
        {
            None => Ok(GetAttachmentDataResponse::ObjectNotFound),
            Some(metadata) =>
            {
                let metadata = data::get::AttachmentMetadata
                {
                    filename: metadata.filename,
                    created: data::Date::from_db_timestamp(metadata.created),
                    modified: data::Date::from_db_timestamp(metadata.modified),
                    mime: metadata.mime.parse::<mime::Mime>()?,
                    size: metadata.size as u64,
                    hash: metadata.hash,
                };

                if self.specific_hash.is_none()
                    || (*self.specific_hash.as_ref().unwrap() == metadata.hash)
                {
                    let bytes = ops.get_attachment_data(&self.object_id.0)?
                        .ok_or(Error::DatabaseConsistencyError{ msg: format!("Object {} contains attachment metadata but no attachment data", self.object_id.0) })?;
                    
                    Ok(GetAttachmentDataResponse::Found{metadata, bytes})
                }
                else
                {
                    Ok(GetAttachmentDataResponse::HashNotFound)
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum GetAttachmentDataResponse
{
    ObjectNotFound,
    HashNotFound,
    Found
    {
        metadata: data::get::AttachmentMetadata,
        bytes: Vec<u8>,
    },
}
