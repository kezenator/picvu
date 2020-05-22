pub fn index() -> String { "/".to_owned() }
pub fn index_with_pagination(page: u64, page_size: u64) -> String { format!("/?offset={}&page_size={}", (page - 1) * page_size, page_size) }
pub fn form_add_object() -> String { "/form/add_object".to_owned() }
pub fn form_bulk_import() -> String { "/form/bulk_import".to_owned() }
pub fn form_bulk_acknowledge() -> String { "/form/bulk_acknowledge".to_owned() }
pub fn attachment_data(object_id: &picvudb::data::ObjectId, hash: &String) -> String { format!("/attachments/{}?hash={}", object_id.to_string(), hash) }
pub fn image_thumbnail(object_id: &picvudb::data::ObjectId, hash: &String, size: u32) -> String { format!("/thumbnails/{}?hash={}&size={}", object_id.to_string(), hash, size) }