use horrorshow::prelude::*;
use horrorshow::{owned_html, box_html, Raw};
use actix_web::HttpRequest;

use crate::analyse;
use crate::bulk;
use crate::format;
use crate::path;
use crate::view;
use crate::pages::HeaderLinkCollection;

use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn header(title: String, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Raw<String>
{
    let html = owned_html!{

        div(class="header")
        {
            h1: title;

            div(class="header-links")
            {
                @for header in header_links.by_order()
                {
                    @if header.path == req.path()
                    {
                        a(href=(header.path))
                        {
                            : format!("[[ {} ]]", header.label)
                        }
                    }
                    else
                    {
                        a(href=(header.path))
                        {
                            : header.label
                        }
                    }
                }
            }
        }

    }.into_string().unwrap();

    Raw(html)
}

pub fn objects_thumbnails(resp: GetObjectsResponse, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Page
{
    let title = format::query_to_string(&resp.query);

    let mut cur_heading = String::new();

    let contents = owned_html!{

        : header(format::query_to_string(&resp.query), req, header_links);

        : (pagination(resp.query.clone(), view::derived::ViewObjectsListType::ThumbnailsGrid, resp.pagination_response.clone()));

        div(class="object-listing")
        {
            @for object in resp.objects.iter()
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

pub fn objects_details(resp: GetObjectsResponse, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Page
{
    let now = picvudb::data::Date::now();

    let title = format::query_to_string(&resp.query);

    let contents = owned_html!{

        : header(format::query_to_string(&resp.query), req, header_links);

        : (pagination(resp.query.clone(), view::derived::ViewObjectsListType::DetailsTable, resp.pagination_response.clone()));

        table(class="details-table")
        {
            tr
            {
                th: "ID";
                th: "Title";
                th: "Activity";
                th: "Size";
                th: "Mime";
            }

            @for object in resp.objects.iter()
            {
                tr
                {
                    td
                    {
                        a(href=path::object_details(&object.id))
                        {
                            : object.id.to_string();
                        }
                    }

                    td: object.title.clone().unwrap_or_default();
                    td: format::date_to_str(&object.activity_time, &now);

                    @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
                    {
                        td: format::bytes_to_string(photo.attachment.size);
                        td: photo.attachment.mime.to_string();
                    }
                    else if let picvudb::data::get::AdditionalMetadata::Video(video) = &object.additional
                    {
                        td: format::bytes_to_string(video.attachment.size);
                        td: video.attachment.mime.to_string();
                    }
                    else
                    {
                        td: "N/A";
                        td: "N/A";
                    }
                }
            }
        }

    }.into_string().unwrap();

    Page {
        title,
        contents,
    }
}

pub fn object_details(object: picvudb::data::get::ObjectMetadata, image_analysis: Result<Option<(analyse::img::ImgAnalysis, Vec<String>)>, analyse::img::ImgAnalysisError>, mvimg_split: analyse::img::MvImgSplit, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Page
{
    let now = picvudb::data::Date::now();

    let title = object.title.clone().unwrap_or(format!("Object {}", object.id.to_string()));
    let title_for_heading = title.clone();

    let contents = owned_html!
    {
        : header(title_for_heading, req, header_links);

        table(class="details-table")
        {
            @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
            {
                tr
                {
                    th(colspan="2"): "Preview";
                }
                tr
                {
                    td(colspan="2")
                    {
                        a(href=path::attachment_data(&object.id, &photo.attachment.hash))
                        {
                            img(src=path::image_thumbnail(&object.id, &photo.attachment.hash, 512))
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
                td: format::date_to_str(&object.created_time, &now);
            }
            tr
            {
                td: "Modified";
                td: format::date_to_str(&object.modified_time, &now);
            }
            tr
            {
                td: "Activity";
                td: format::date_to_str(&object.activity_time, &now);
            }
            tr
            {
                td: "Type";
                td: object.obj_type.to_string();
            }
            @if object.title.is_some()
            {
                tr
                {
                    td: "Title";
                    td: object.title.clone().unwrap_or(String::new());
                }
            }
            @if object.notes.is_some()
            {
                tr
                {
                    td: "Notes";
                    td: object.notes.clone().unwrap_or(String::new());
                }
            }

            : location_details(&object.location);

            @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
            {
                : attachment_details(&object.id, &photo.attachment, &mvimg_split, &now);
            }
            else if let picvudb::data::get::AdditionalMetadata::Video(video) = &object.additional
            {
                : attachment_details(&object.id, &video.attachment, &mvimg_split, &now);
            }

            : exif_details(&image_analysis);
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

                @if let Some(orig_taken) = image_analysis.orig_taken
                {
                    tr
                    {
                        td: "Taken";
                        td: format::date_to_str(&orig_taken, &now);
                    }
                }

                @if image_analysis.orig_taken_naive.is_some()
                    || image_analysis.orig_digitized_naive.is_some()
                    || image_analysis.gps_timestamp.is_some()
                {
                    tr
                    {
                        td: "Timestamps";
                        td
                        {
                            @if let Some(taken) = image_analysis.orig_taken_naive
                            {
                                p: format!("Orig Taken: {:?}", taken);
                            }
                            @if let Some(digitized) = image_analysis.orig_digitized_naive
                            {
                                p: format!("Digitized: {:?}", digitized);
                            }
                            @if let Some(gps) = image_analysis.gps_timestamp
                            {
                                p: format!("GPS: {:?}", gps);
                            }
                        }
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

fn attachment_details(obj_id: &picvudb::data::ObjectId, attachment: &picvudb::data::get::AttachmentMetadata, mvimg_split: &analyse::img::MvImgSplit, now: &picvudb::data::Date) -> Box<dyn RenderBox>
{
    let now = now.clone();
    let obj_id = obj_id.clone();
    let file_name = attachment.filename.clone();
    let created = attachment.created.clone();
    let modified = attachment.modified.clone();
    let size = attachment.size.clone();
    let mime = attachment.mime.clone();
    let hash = attachment.hash.clone();
    let mvimg_split = mvimg_split.clone();

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
                p
                {
                    a(href=path::attachment_data(&obj_id, &hash)): "View";
                }

                @if let analyse::img::MvImgSplit::Mp4Only = mvimg_split
                {
                    p
                    {
                        a(href=path::attachment_as_mp4(&obj_id, &hash, 0))
                        {
                            : "This file is actually a MP4 movie, not a JPEG image.";
                        }
                    }
                }
                else if let analyse::img::MvImgSplit::Both{mp4_offset} = mvimg_split
                {
                    p
                    {
                        a(href=path::attachment_as_mp4(&obj_id, &hash, mp4_offset))
                        {
                            : format!("This file contains a {} JPEG image then a {} MP4 movie",
                                format::bytes_to_string(mp4_offset as u64),
                                format::bytes_to_string(size - (mp4_offset as u64)));
                        }
                    }
                }
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

fn pagination(query: picvudb::data::get::GetObjectsQuery, list_type: view::derived::ViewObjectsListType, response: picvudb::data::get::PaginationResponse) -> Raw<String>
{
    let this_page_offset = response.offset;
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

    let result: String = owned_html!
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
                        a(href=path::objects_with_options(query.clone(), list_type, (*page - 1) * page_size, page_size))
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
                : (format!("Total: {} objects ", total));

                a(href=path::objects_with_options(query.clone(), view::derived::ViewObjectsListType::ThumbnailsGrid, this_page_offset, page_size))
                {
                    @if list_type == view::derived::ViewObjectsListType::ThumbnailsGrid
                    {
                        : " [[ Thumbnails ]] ";
                    }
                    else
                    {
                        : " Thumbnails ";
                    }
                }

                a(href=path::objects_with_options(query.clone(), view::derived::ViewObjectsListType::DetailsTable, this_page_offset, page_size))
                {
                    @if list_type == view::derived::ViewObjectsListType::DetailsTable
                    {
                        : "[[ Details ]]";
                    }
                    else
                    {
                        : " Details ";
                    }
                }
            }
        }
    }.into_string().unwrap();

    Raw(result)
}
