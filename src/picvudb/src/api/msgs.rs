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
pub struct SetPropertiesRequest
{
    pub properties: HashMap<String, String>,
}

impl ApiMessage for SetPropertiesRequest
{
    type Response = SetPropertiesResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        ops.set_properties(&self.properties)?;

        Ok(SetPropertiesResponse{})
    }
}

#[derive(Debug)]
pub struct SetPropertiesResponse
{
}

#[derive(Debug)]
pub struct GetStatisticsRequest
{
}

impl ApiMessage for GetStatisticsRequest
{
    type Response = GetStatisticsResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let num_objects = ops.get_num_objects()?;

        Ok(GetStatisticsResponse{ num_objects })
    }
}

#[derive(Debug)]
pub struct GetStatisticsResponse
{
    pub num_objects: u64,
}

#[derive(Debug)]
pub struct GetObjectsRequest
{
    pub query: data::get::GetObjectsQuery,
    pub pagination: data::get::PaginationRequest,
}

impl ApiMessage for GetObjectsRequest
{
    type Response = GetObjectsResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let mut results = Vec::new();

        let num_objects = match self.query
        {
            data::get::GetObjectsQuery::ByActivityDesc 
                | data::get::GetObjectsQuery::ByModifiedDesc => ops.get_num_objects()?,
            data::get::GetObjectsQuery::ByAttachmentSizeDesc => ops.get_num_objects_with_attachments()?,
            data::get::GetObjectsQuery::ByObjectId(_) => 1,
        };

        // Fix up the pagination request
        let mut pagination = self.pagination.clone();
        {
            if pagination.page_size < 10
            {
                pagination.page_size = 10;
            }
            if pagination.offset >= num_objects
            {
                if num_objects == 0
                {
                    pagination.offset = 0;
                }
                else
                {
                    pagination.offset = num_objects - 1;
                }
            }
            pagination.offset /= pagination.page_size;
            pagination.offset *= pagination.page_size;
        }

        let mut from_db = match &self.query
        {
            data::get::GetObjectsQuery::ByActivityDesc => ops.get_objects_by_activity_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByModifiedDesc => ops.get_objects_by_modified_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByAttachmentSizeDesc => ops.get_objects_by_attachment_size_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByObjectId(obj_id) => ops.get_object_by_id(obj_id.to_string())?.iter().map(|o| { o.clone() }).collect(),
        };

        results.reserve(from_db.len());

        for object in from_db.drain(..)
        {
            let obj_type = data::ObjectType::from_db_string(&object.obj_type)?;

            let additional = match obj_type
            {
                data::ObjectType::Photo =>
                {
                    if let Some(attachment) = ops.get_attachment_metadata(&object.id)?
                    {
                        data::get::AdditionalMetadata::Photo(data::get::PhotoMetadata
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
                        })
                    }
                    else
                    {
                        return Err(Error::DatabaseConsistencyError
                        {
                            msg: format!("Object {} is a photo but contains no attachment metadata", object.id.to_string()),
                        });
                    }
                },
                data::ObjectType::Video =>
                {
                    if let Some(attachment) = ops.get_attachment_metadata(&object.id)?
                    {
                        data::get::AdditionalMetadata::Video(data::get::VideoMetadata
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
                        })
                    }
                    else
                    {
                        return Err(Error::DatabaseConsistencyError
                        {
                            msg: format!("Object {} is a video but contains no attachment metadata", object.id.to_string()),
                        });
                    }
                },
            };

            let location = match (object.latitude, object.longitude)
            {
                (Some(latitude), Some(longitude)) => Some(data::Location::new(latitude, longitude, None)),
                _ => None
            };

            results.push(data::get::ObjectMetadata
            {
                id: data::ObjectId::new(object.id),
                created_time: data::Date::from_db_fields(object.created_timestamp, object.created_timestring),
                modified_time: data::Date::from_db_fields(object.modified_timestamp, object.modified_timestring),
                activity_time: data::Date::from_db_fields(object.activity_timestamp, object.activity_timestring),
                obj_type: obj_type,
                title: object.title,
                notes: object.notes,
                location: location,
                additional: additional,
            });
        }

        Ok(GetObjectsResponse
            {
                objects: results,
                query: self.query.clone(),
                pagination_request: self.pagination.clone(),
                pagination_response: data::get::PaginationResponse{
                    offset: pagination.offset,
                    page_size: pagination.page_size,
                    total: num_objects,
                }
            })
    }
}

#[derive(Debug)]
pub struct GetObjectsResponse
{
    pub objects: Vec<data::get::ObjectMetadata>,
    pub query: data::get::GetObjectsQuery,
    pub pagination_request: data::get::PaginationRequest,
    pub pagination_response: data::get::PaginationResponse,
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
        let obj_type = match &self.data.additional
        {
            data::add::AdditionalData::Photo{..} => data::ObjectType::Photo,
            data::add::AdditionalData::Video{..} => data::ObjectType::Video,
        };

        let object = ops.add_object(
            obj_type,
            self.data.created_time.clone(),
            self.data.activity_time.clone(),
            self.data.title.clone(),
            self.data.notes.clone(),
            self.data.location.clone())?;

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
            data::add::AdditionalData::Video{attachment} =>
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
