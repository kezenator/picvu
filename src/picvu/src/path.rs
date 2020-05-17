pub fn index() -> String { "/".to_owned() }
pub fn form_add_object() -> String { "/form/add_object".to_owned() }
pub fn attachment_data(object_id: &picvudb::data::ObjectId) -> String { format!("/attachments/{}", object_id.to_string()) }