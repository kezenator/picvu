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
            tr { th : "ID"; th: "Added"; th: "Changed"; th: "Label" }
            @ for object in resp.objects.iter()
            {
                tr
                {
                    td: object.id.clone();
                    td: object.added.to_rfc3339();
                    td: object.changed.to_rfc3339();
                    td: object.label.clone();
                }
            }
        }
        h1: "Add New Object";
        form(method="POST", action=path::form_add_object(), enctype="application/x-www-form-urlencoded")
        {
            input(type="text", name="label");
            input(type="submit");
        }
    }.to_string();

    Page {
        title: "All Objects".to_owned(),
        contents: contents,
    }
}