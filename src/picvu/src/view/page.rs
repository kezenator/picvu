use horrorshow::html;

use crate::path;
use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn all_objects(resp: &GetAllObjectsResponse) -> Page
{
    let contents = html!{
        h1: "All Objects";
        table
        {
            tr
            {
                th : "ID";
                th: "Added";
                th: "Changed";
                th: "Title";
                th: "Size (bytes)";
                th: "Preview";
            }

            @ for object in resp.objects.iter()
            {
                tr
                {
                    td: object.id.to_string();
                    td: object.added.to_rfc3339();
                    td: object.changed.to_rfc3339();
                    td: object.title.clone().unwrap_or(String::new());

                    @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
                    {
                        td: photo.attachment.size;
                        td
                        {
                            img(src=path::attachment_data(&object.id))
                        }
                    }
                    else
                    {
                        td: "N/A";
                        td: "N/A";
                    }
                }
            }
        }
        h1: "Add New Object";
        form(method="POST", action=path::form_add_object(), enctype="multipart/form-data")
        {
            input(type="file", name="file", accept="image/*,video/*");
            input(type="submit");
        }
    }.to_string();

    Page {
        title: "All Objects".to_owned(),
        contents: contents,
    }
}