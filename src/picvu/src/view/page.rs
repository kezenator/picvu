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
                            a(href=path::attachment_data(&object.id, &photo.attachment.hash))
                            {
                                img(src=path::image_thumbnail(&object.id, &photo.attachment.hash, 128))
                            }
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
        h1: "Bulk Import";
        form(method="POST", action=path::form_bulk_import(), enctype="application/x-www-form-urlencoded")
        {
            input(type="text", name="folder");
            input(type="submit");
        }
    }.to_string();

    Page {
        title: "All Objects".to_owned(),
        contents: contents,
    }
}