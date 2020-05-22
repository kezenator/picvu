use horrorshow::{html, owned_html};

use crate::path;
use crate::bulk;
use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn all_objects(resp: GetAllObjectsResponse) -> Page
{
    let contents = html!{
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
                    else if let picvudb::data::get::AdditionalMetadata::Video(video) = &object.additional
                    {
                        td: video.attachment.size;
                        td
                        {
                            a(href=path::attachment_data(&object.id, &video.attachment.hash))
                            {
                                : "Video link"
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

pub fn bulk_progress(progress: bulk::progress::ProgressState) -> Page
{
    let contents = owned_html!{
        ol
        {
            @for stage in progress.completed_stages.iter()
            {
                li { p : (stage) }
            }

            li
            {
                p : progress.current_stage.clone();

                ul
                {
                    p : (format!("{:.1}", progress.percentage_complete));
                    
                    @for line in progress.progress_lines.iter()
                    {
                        p : line;
                    }
                }
            }

            @for stage in progress.remaining_stages.iter()
            {
                li { p : (stage) }
            }

            @if progress.complete
            {
                form(method="POST", action=path::form_bulk_acknowledge(), enctype="application/x-www-form-urlencoded")
                {
                    input(type="submit", value="Acknowledge");
                }
            }
        }

        @if !progress.complete
        {
            script
            {
                : (horrorshow::Raw("window.setTimeout(\"window.location.reload();\", 1000);"))
            }
        }
        
    }.to_string();

    Page {
        title: "Bulk Operations".to_owned(),
        contents: contents,
    }
}