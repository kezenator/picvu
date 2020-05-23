use horrorshow::prelude::*;
use horrorshow::{html, owned_html, box_html};

use crate::path;
use crate::bulk;
use crate::format;
use crate::view::derived;
use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn objects(resp: GetObjectsResponse) -> Page
{
    let contents = html!{

        p
        {
            a(href=(path::index())) : "All Objects";
            : ", ";
            a(href=(path::objects(picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc))) : "By Size";
        }

        : (pagination(resp.query.clone(), resp.pagination_response.clone()));

        div
        {
            @ for object in resp.objects.iter()
            {
                span
                {
                    p
                    {
                        @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
                        {
                            a(href=path::object_details(&object.id))
                            {
                                img(src=path::image_thumbnail(&object.id, &photo.attachment.hash, 128))
                            }
                        }
                        else if let picvudb::data::get::AdditionalMetadata::Video(_video) = &object.additional
                        {
                            a(href=path::object_details(&object.id))
                            {
                                : "Video"
                            }
                        }
                        else
                        {
                            a(href=path::object_details(&object.id))
                            {
                                : "Other"
                            }
                        }
                    }
                    p: object.title.clone().unwrap_or(String::new());
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

pub fn object_details(view: &derived::ViewObjectDetails) -> Page
{
    let now = picvudb::data::Date::now();
    let title = view.object.title.clone().unwrap_or(format!("Object {}", view.object.id.to_string()));

    let contents = html!
    {
        table
        {
            @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &view.object.additional
            {
                tr
                {
                    th(colspan="2"): "Preview";
                }
                tr
                {
                    td(colspan="2")
                    {
                        a(href=path::attachment_data(&view.object.id, &photo.attachment.hash))
                        {
                            img(src=path::image_thumbnail(&view.object.id, &photo.attachment.hash, 512))
                        }
                    }
                }
            }

            tr
            {
                th(colspan="2"): "Details";
            }
            tr
            {
                td: "Created";
                td: format::date_to_str(&view.object.added, &now);
            }
            tr
            {
                td: "Modified";
                td: format::date_to_str(&view.object.changed, &now);
            }
            tr
            {
                td: "Type";
                td: view.object.obj_type.to_string();
            }
            @if view.object.title.is_some()
            {
                tr
                {
                    td: "Title";
                    td: view.object.title.clone().unwrap_or(String::new());
                }
            }

            @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &view.object.additional
            {
                : attachment_details(&view.object.id, &photo.attachment, &now);
            }
            else if let picvudb::data::get::AdditionalMetadata::Video(video) = &view.object.additional
            {
                : attachment_details(&view.object.id, &video.attachment, &now);
            }

            @if let Ok(image_analysis) = view.image_analysis.clone()
            {
                @if let Some(image_analysis) = image_analysis
                {
                    tr
                    {
                        th(colspan="2"): "Photo EXIF Data";
                    }

                    tr
                    {
                        td: "Orientation";
                        td: image_analysis.orientation.to_string();
                    }

                    @if let Some(original_datetime) = image_analysis.original_datetime
                    {
                        tr
                        {
                            td: "Taken";
                            td: format::date_to_str(&original_datetime, &now);
                        }
                    }

                    @if let Some(exposure) = image_analysis.exposure
                    {
                        tr
                        {
                            td: "Camera Settings";
                            td: format!("{} {} {}", exposure.aperture, exposure.time, exposure.iso);
                        }
                    }

                    @if let Some(location) = image_analysis.location
                    {
                        tr
                        {
                            td: "Location";
                            td
                            {
                                a(href=format!("https://www.google.com/maps/search/?api=1&query={},{}", location.latitude, location.longitude),
                                    target="_blank")
                                {
                                    : format!("{}, {}", location.latitude, location.longitude);
                                }
                            }
                        }
                        tr
                        {
                            td: "Altitude";
                            td: format!("{:.0} m", location.altitude_meters);
                        }
                    }
                }
            }
            else if let Err(image_analysis_err) = view.image_analysis.clone()
            {
                tr
                {
                    th(colspan="2"): "Photo EXIF Data";
                }

                tr
                {
                    td: "Error";
                    td: image_analysis_err.msg;
                }
            }
        }
    }.to_string();

    Page
    {
        title,
        contents,
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

fn attachment_details(obj_id: &picvudb::data::ObjectId, attachment: &picvudb::data::get::AttachmentMetadata, now: &picvudb::data::Date) -> Box<dyn RenderBox>
{
    let now = now.clone();
    let obj_id = obj_id.clone();
    let created = attachment.created.clone();
    let modified = attachment.modified.clone();
    let size = attachment.size.clone();
    let mime = attachment.mime.clone();
    let hash = attachment.hash.clone();

    box_html!
    {
        tr
        {
            td(colspan="2"): "Attachment";
        }
        tr
        {
            td: "Created";
            td: format::date_to_str(&created, &now);
        }
        tr
        {
            td: "Modified";
            td: format::date_to_str(&modified, &now);
        }
        tr
        {
            td: "Size";
            td: format::bytes_to_str(size);
        }
        tr
        {
            td: "Mime Type";
            td: mime.to_string();
        }
        tr
        {
            td: "Hash";
            td: hash.clone();
        }
        tr
        {
            td: "Link";
            td
            {
                a(href=path::attachment_data(&obj_id, &hash)): "View";
            }
        }
    }
}

fn should_print_page(page: u64, cur_page: u64, last_page: u64) -> bool
{
    if page <= 3
    {
        return true;
    }
    else if (page + 3) >= last_page
    {
        return true;
    }
    else if (page <= cur_page)
        && ((cur_page - page) <= 3)
    {
        return true;
    }
    else if (page >= cur_page)
        && ((page - cur_page) <= 3)
    {
        return true;
    }
    else
    {
        return false;
    }
}

fn pagination(query: picvudb::data::get::GetObjectsQuery, response: picvudb::data::get::PaginationResponse) -> Box<dyn RenderBox>
{
    let page_size = response.page_size;
    let total = response.total;

    let mut pages = Vec::new();
    {
        let mut offset = 0;
        let mut page = 1;
        while offset < response.total
        {
            pages.push(page);
            offset += response.page_size;
            page += 1;
        }
        if pages.is_empty()
        {
            pages.push(1);
        }
    };
    let mut done_elipsis = false;

    let cur_page = (response.offset / response.page_size) + 1;
    let last_page = *pages.last().unwrap();

    box_html!
    {
        p
        {
            @for page in pages.iter()
            {
                @if should_print_page(*page, cur_page, last_page)
                {
                    : ({ done_elipsis = false; ""});
                    a(href=path::objects_with_pagination(query.clone(), (*page - 1) * page_size, page_size)): (format!("{}, ", page));
                }
                else
                {
                    @if !done_elipsis
                    {
                        : ({ done_elipsis = true; " ... " });
                    }
                }
            }

            : (format!("Total: {} objects", total));
        }
    }
}
