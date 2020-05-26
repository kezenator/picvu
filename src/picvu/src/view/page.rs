use horrorshow::prelude::*;
use horrorshow::{owned_html, box_html};

use crate::analyse;
use crate::bulk;
use crate::format;
use crate::path;
use crate::view::derived;
use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn objects(resp: GetObjectsResponse) -> Page
{
    let title = format::query_to_string(&resp.query);

    let mut cur_heading = String::new();

    let contents = owned_html!{

        div(class="header")
        {
            h1: format::query_to_string(&resp.query);

            div(class="header-links")
            {
                a(href=(path::index()))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByActivityDesc);
                }
                a(href=(path::objects(picvudb::data::get::GetObjectsQuery::ByModifiedDesc)))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByModifiedDesc);
                }
                a(href=(path::objects(picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc)))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc);
                }
            }
        }

        : (pagination(resp.query.clone(), resp.pagination_response.clone()));

        div(class="object-listing")
        {
            @ for object in resp.objects.iter()
            {
                @if let this_heading = get_heading(object, &resp.query)
                {
                    @if this_heading != cur_heading
                    {
                        h2(class="object-listing-group")
                        {
                            : ({ cur_heading = this_heading; cur_heading.clone() });
                        }
                    }
                }

                div(class="object-listing-entry")
                {
                    div(class="object-listing-thumbnail")
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
                    div(class="object-listing-title")
                    {
                        : format::insert_zero_width_spaces(object.title.clone().unwrap_or(String::new()));
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
    }.into_string().unwrap();

    Page {
        title,
        contents,
    }
}

pub fn object_details(view: &derived::ViewObjectDetails) -> Page
{
    let now = picvudb::data::Date::now();

    let title = view.object.title.clone().unwrap_or(format!("Object {}", view.object.id.to_string()));
    let title_for_heading = title.clone();

    let contents = owned_html!
    {
        div(class="header")
        {
            h1: title_for_heading;

            div(class="header-links")
            {
                a(href=(path::index()))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByActivityDesc);
                }
                a(href=(path::objects(picvudb::data::get::GetObjectsQuery::ByModifiedDesc)))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByModifiedDesc);
                }
                a(href=(path::objects(picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc)))
                {
                    : format::query_to_string(&picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc);
                }
            }
        }

        table(class="details-table")
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
                td: format::date_to_str(&view.object.created_time, &now);
            }
            tr
            {
                td: "Modified";
                td: format::date_to_str(&view.object.modified_time, &now);
            }
            tr
            {
                td: "Activity";
                td: format::date_to_str(&view.object.activity_time, &now);
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
            @if view.object.notes.is_some()
            {
                tr
                {
                    td: "Notes";
                    td: view.object.notes.clone().unwrap_or(String::new());
                }
            }

            : location_details(&view.object.location);

            @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &view.object.additional
            {
                : attachment_details(&view.object.id, &photo.attachment, &now);
            }
            else if let picvudb::data::get::AdditionalMetadata::Video(video) = &view.object.additional
            {
                : attachment_details(&view.object.id, &video.attachment, &now);
            }

            : exif_details(&view.image_analysis);
        }
    }.into_string().unwrap();

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
                    p : (format!("{:.1}%", progress.percentage_complete));
                    
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
        
    }.into_string().unwrap();

    Page {
        title: "Bulk Operations".to_owned(),
        contents: contents,
    }
}

fn get_heading(object: &picvudb::data::get::ObjectMetadata, query: &picvudb::data::get::GetObjectsQuery) -> String
{
    match query
    {
        picvudb::data::get::GetObjectsQuery::ByObjectId(_) =>
        {
            object.id.to_string()
        },
        picvudb::data::get::GetObjectsQuery::ByModifiedDesc =>
        {
            format::date_to_date_only_string(&object.modified_time)
        },
        picvudb::data::get::GetObjectsQuery::ByActivityDesc =>
        {
            format::date_to_date_only_string(&object.activity_time)
        },
        picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc =>
        {
            let size = match &object.additional
            {
                picvudb::data::get::AdditionalMetadata::Photo(photo) =>
                {
                    photo.attachment.size
                },
                picvudb::data::get::AdditionalMetadata::Video(video) =>
                {
                    video.attachment.size
                },
            };

            format::bytes_to_group_header(size)
        },
    }
}

fn exif_details(exif: &Result<Option<(analyse::img::ImgAnalysis, Vec<String>)>, analyse::img::ImgAnalysisError>) -> Box<dyn RenderBox>
{
    let exif = exif.clone();
    let now = picvudb::data::Date::now();

    box_html!
    {
        @if let Ok(image_analysis) = exif
        {
            @if let Some((image_analysis, exif_warnings)) = image_analysis
            {
                tr
                {
                    th(colspan="2"): "Photo EXIF Data";
                }

                @if let Some(orientation) = image_analysis.orientation
                {
                    tr
                    {
                        td: "Orientation";
                        td: orientation.to_string();
                    }
                }

                @if let Some(make_model) = image_analysis.make_model
                {
                    tr
                    {
                        td: "Model";
                        td: format!("{} {}", make_model.make, make_model.model);
                    }
                }

                @if let Some(original_datetime) = image_analysis.original_datetime
                {
                    tr
                    {
                        td: "Taken";
                        td: format::date_to_str(&original_datetime, &now);
                    }
                }

                @if let Some(camera_settings) = image_analysis.camera_settings
                {
                    tr
                    {
                        td: "Camera Settings";
                        td: format!("{} {} {} {}",
                            camera_settings.aperture,
                            camera_settings.exposure_time,
                            camera_settings.focal_length,
                            camera_settings.iso);
                    }
                }

                : location_details(&image_analysis.location);

                @if let Some(dop) = image_analysis.location_dop
                {
                    tr
                    {
                        td: "Location DOP";
                        td: format!("{:.1}", dop);
                    }
                }

                @for w in exif_warnings
                {
                    tr
                    {
                        td: "Warning";
                        td
                        {
                            : w;
                        }
                    }
                }
            }
        }
        else if let Err(image_analysis_err) = exif
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
}

fn location_details(location: &Option<picvudb::data::Location>) -> Box<dyn RenderBox>
{
    let location = location.clone();

    box_html!
    {
        @if let Some(location) = location
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

            @if let Some(altitude) = location.altitude
            {
                tr
                {
                    td: "Altitude";
                    td: format!("{:.0} m", altitude);
                }
            }
        }
    }
}

fn attachment_details(obj_id: &picvudb::data::ObjectId, attachment: &picvudb::data::get::AttachmentMetadata, now: &picvudb::data::Date) -> Box<dyn RenderBox>
{
    let now = now.clone();
    let obj_id = obj_id.clone();
    let file_name = attachment.filename.clone();
    let created = attachment.created.clone();
    let modified = attachment.modified.clone();
    let size = attachment.size.clone();
    let mime = attachment.mime.clone();
    let hash = attachment.hash.clone();

    box_html!
    {
        tr
        {
            th(colspan="2"): "Attachment";
        }
        tr
        {
            td: "File Name";
            td: file_name;
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
            td: format::bytes_to_string(size);
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
        div(class="pagination")
        {
            @for page in pages.iter()
            {
                @if should_print_page(*page, cur_page, last_page)
                {
                    : ({ done_elipsis = false; ""});
                    div(class="pagintation-link")
                    {
                        a(href=path::objects_with_pagination(query.clone(), (*page - 1) * page_size, page_size))
                        {
                            : (format!("{}, ", page));
                        }
                    }
                }
                else
                {
                    @if !done_elipsis
                    {
                        div(class="pagination-elipsis")
                        {
                            : ({ done_elipsis = true; "..." });
                        }
                    }
                }
            }

            div(class="pagination-summary")
            {
                : (format!("Total: {} objects", total));
            }
        }
    }
}
