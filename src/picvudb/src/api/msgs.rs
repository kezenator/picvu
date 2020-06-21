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
pub struct GetNumObjectsRequest
{
    pub query: data::get::GetObjectsQuery,
}

impl ApiMessage for GetNumObjectsRequest
{
    type Response = GetNumObjectsResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        let num_objects = match &self.query
        {
            data::get::GetObjectsQuery::ByActivityDesc 
                | data::get::GetObjectsQuery::ByModifiedDesc => ops.get_num_objects()?,
            data::get::GetObjectsQuery::ByAttachmentSizeDesc => ops.get_num_objects_with_attachments()?,
            data::get::GetObjectsQuery::ByObjectId(_) => 1,
            data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters } => ops.get_num_objects_near_location(location.latitude, location.longitude, *radius_meters)?,
            data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ search } => ops.get_num_objects_for_text_search(search)?,
            data::get::GetObjectsQuery::TagByActivityDesc{ tag_id } => ops.get_num_objects_with_tag(tag_id.to_db_field())?,
        };

        let response = GetNumObjectsResponse
        {
            query: self.query.clone(),
            num_objects: num_objects,
        };

        Ok(response)
    }
}

#[derive(Debug)]
pub struct GetNumObjectsResponse
{
    pub query: data::get::GetObjectsQuery,
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
        let num_objects = GetNumObjectsRequest{ query: self.query.clone() }
            .execute(ops)?
            .num_objects;

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

        let mut results = Vec::new();

        let mut from_db = match &self.query
        {
            data::get::GetObjectsQuery::ByActivityDesc => ops.get_objects_by_activity_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByModifiedDesc => ops.get_objects_by_modified_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByAttachmentSizeDesc => ops.get_objects_by_attachment_size_desc(pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::ByObjectId(obj_id) => ops.get_object_by_id(obj_id.to_db_field())?.iter().map(|o| { o.clone() }).collect(),
            data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters } => ops.get_objects_near_location_by_activity_desc(location.latitude, location.longitude, *radius_meters, pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ search } => ops.get_objects_for_text_search(search, pagination.offset, pagination.page_size)?,
            data::get::GetObjectsQuery::TagByActivityDesc{ tag_id } => ops.get_objects_with_tag_by_activity_desc(tag_id.to_db_field(), pagination.offset, pagination.page_size)?,
        };

        results.reserve(from_db.len());

        for object in from_db.drain(..)
        {
            let attachment = 
            {
                if let Some(attachment) = ops.get_attachment_metadata(object.id)?
                {
                    data::get::AttachmentMetadata
                    {
                        filename: attachment.filename,
                        created: data::Date::from_db_fields(attachment.created_timestamp, attachment.created_offset)?,
                        modified: data::Date::from_db_fields(attachment.modified_timestamp, attachment.modified_offset)?,
                        mime: attachment.mime.parse::<mime::Mime>()?,
                        size: attachment.size as u64,
                        orientation: data::Orientation::from_db_field(attachment.orientation)?,
                        dimensions: data::Dimensions::from_db_fields(attachment.width, attachment.height),
                        duration: data::Duration::from_db_field(attachment.duration)?,
                        hash: attachment.hash,
                    }
                }
                else
                {
                    return Err(Error::DatabaseConsistencyError
                    {
                        msg: format!("Object {} contains no attachment metadata", object.id.to_string()),
                    });
                }
            };

            let location = match (object.latitude, object.longitude)
            {
                (Some(latitude), Some(longitude)) => Some(data::Location::new(latitude, longitude, None)),
                _ => None
            };

            let mut tags = Vec::new();
            {
                let tag_set = data::TagSet::from_db_field(object.tag_set)?;

                tags.reserve(tag_set.to_db_vec().len());

                for tag_id in tag_set.to_db_vec()
                {
                    let tag_data = ops.get_tag(*tag_id)?;

                    tags.push(data::get::TagMetadata {
                        tag_id: data::TagId::from_db_field(tag_data.tag_id),
                        name: tag_data.tag_name,
                        kind: data::TagKind::from_db_field(tag_data.tag_kind)?,
                        rating: data::Rating::from_db_field(tag_data.tag_rating)?,
                        censor: data::Censor::from_db_field(tag_data.tag_censor)?,
                    });
                }
            }

            results.push(data::get::ObjectMetadata
            {
                id: data::ObjectId::from_db_field(object.id),
                created_time: data::Date::from_db_fields(object.created_timestamp, object.created_offset)?,
                modified_time: data::Date::from_db_fields(object.modified_timestamp, object.modified_offset)?,
                activity_time: data::Date::from_db_fields(object.activity_timestamp, object.activity_offset)?,
                title: object.title,
                notes: object.notes,
                rating: data::Rating::from_db_field(object.rating)?,
                censor: data::Censor::from_db_field(object.censor)?,
                location: location,
                attachment: attachment,
                tags: tags,
                ext_ref: data::ExternalReference::from_db_fields(object.ext_ref_type, object.ext_ref_id)?,
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
        let mut tag_ids = std::collections::BTreeSet::new();

        for t in self.data.tags.iter()
        {
            let tag_id = ops.find_or_add_tag(
                t.name.clone(),
                t.kind.clone(),
                t.rating.clone(),
                t.censor.clone())?;

            tag_ids.insert(tag_id);
        }

        let object = ops.add_object(
            self.data.created_time.clone(),
            self.data.activity_time.clone(),
            self.data.title.clone(),
            self.data.notes.clone(),
            self.data.rating.clone(),
            self.data.censor.clone(),
            self.data.location.clone(),
            data::TagSet::from_db_set(&tag_ids),
            self.data.ext_ref.clone())?;

        ops.add_attachment(
            object.id,
            self.data.attachment.filename.clone(),
            self.data.attachment.created.clone(),
            self.data.attachment.modified.clone(),
            self.data.attachment.mime.to_string(),
            self.data.attachment.orientation.clone(),
            self.data.attachment.dimensions.clone(),
            self.data.attachment.duration.clone(),
            self.data.attachment.bytes.clone())?;

        for tag_id in tag_ids
        {
            ops.add_object_tag(object.id, tag_id)?;
        }

        Ok(AddObjectResponse{ object_id: data::ObjectId::from_db_field(object.id) })
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
        let metadata = ops.get_attachment_metadata(self.object_id.to_db_field())?;
        match metadata
        {
            None => Ok(GetAttachmentDataResponse::ObjectNotFound),
            Some(metadata) =>
            {
                let metadata = data::get::AttachmentMetadata
                {
                    filename: metadata.filename,
                    created: data::Date::from_db_fields(metadata.created_timestamp, metadata.created_offset)?,
                    modified: data::Date::from_db_fields(metadata.modified_timestamp, metadata.modified_offset)?,
                    mime: metadata.mime.parse::<mime::Mime>()?,
                    size: metadata.size as u64,
                    orientation: data::Orientation::from_db_field(metadata.orientation)?,
                    dimensions: data::Dimensions::from_db_fields(metadata.width, metadata.height),
                    duration: data::Duration::from_db_field(metadata.duration)?,
                    hash: metadata.hash,
                };

                if self.specific_hash.is_none()
                    || (*self.specific_hash.as_ref().unwrap() == metadata.hash)
                {
                    let bytes = ops.get_attachment_data(self.object_id.to_db_field())?
                        .ok_or(Error::DatabaseConsistencyError{ msg: format!("Object {} contains attachment metadata but no attachment data", self.object_id.to_db_field()) })?;
                    
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

#[derive(Debug)]
pub struct UpdateObjectRequest
{
    pub object_id: data::ObjectId,
    pub activity_time: data::Date,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<data::Rating>,
    pub censor: data::Censor,
    pub location: Option<data::Location>,
}

impl ApiMessage for UpdateObjectRequest
{
    type Response = UpdateObjectResponse;
    type Error = Error;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>
    {
        ops.update_object(
            self.object_id.to_db_field(),
            self.activity_time.clone(),
            self.title.clone(),
            self.notes.clone(),
            self.rating.clone(),
            self.censor.clone(),
            self.location.clone())?;

        Ok(UpdateObjectResponse{})
    }
}

#[derive(Debug)]
pub struct UpdateObjectResponse
{
}
