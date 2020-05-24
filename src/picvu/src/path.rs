pub fn index() -> String { "/".to_owned() }
pub fn form_add_object() -> String { "/form/add_object".to_owned() }
pub fn form_bulk_import() -> String { "/form/bulk_import".to_owned() }
pub fn form_bulk_acknowledge() -> String { "/form/bulk_acknowledge".to_owned() }
pub fn attachment_data(object_id: &picvudb::data::ObjectId, hash: &String) -> String { format!("/attachments/{}?hash={}", object_id.to_string(), hash) }
pub fn image_thumbnail(object_id: &picvudb::data::ObjectId, hash: &String, size: u32) -> String { format!("/thumbnails/{}?hash={}&size={}", object_id.to_string(), hash, size) }

pub fn object_details(object_id: &picvudb::data::ObjectId) -> String { format!("/view/object/{}", object_id.to_string()) }

pub fn objects(query: picvudb::data::get::GetObjectsQuery) -> String
{
    match query
    {
        picvudb::data::get::GetObjectsQuery::ByActivityDesc => "/view/objects/by_activity_desc".to_owned(),
        picvudb::data::get::GetObjectsQuery::ByModifiedDesc => "/view/objects/by_modified_desc".to_owned(),
        picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc => "/view/objects/by_size_desc".to_owned(),
        picvudb::data::get::GetObjectsQuery::ByObjectId(obj_id) => object_details(&obj_id),
    }
}

pub fn objects_with_pagination(query: picvudb::data::get::GetObjectsQuery, offset: u64, page_size: u64) -> String
{
    format!("{}?offset={}&page_size={}", objects(query), offset, page_size)
}
