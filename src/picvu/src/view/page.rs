use horrorshow::html;

use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn properties(props: &GetPropertiesResponse) -> Page
{
    let contents = html!{
        table
        {
            tr { th : "Name"; th: "Value" }
            @ for (name, value) in props.properties.iter()
            {
                tr
                {
                    td: name;
                    td: value;
                }
            }
        }
    }.to_string();

    Page {
        title: "Properties".to_owned(),
        contents: contents,
    }
}