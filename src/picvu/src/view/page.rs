use horrorshow::prelude::*;
use horrorshow::{html, owned_html, box_html};

use crate::path;
use crate::bulk;
use crate::format;
use picvudb::msgs::*;

pub struct Page
{
    pub title: String,
    pub contents: String,
}

pub fn objects(resp: GetObjectsResponse) -> Page
{
    let now = picvudb::data::Date::now();

    let contents = html!{
        table
        {
            : (pagination(&resp.pagination));

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
                    td: format::date_to_str(&object.added, &now);
                    td: format::date_to_str(&object.changed, &now);
                    td: object.title.clone().unwrap_or(String::new());

                    @if let picvudb::data::get::AdditionalMetadata::Photo(photo) = &object.additional
                    {
                        td: format::bytes_to_str(photo.attachment.size);
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
                        td: format::bytes_to_str(video.attachment.size);
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

fn pagination(response: &picvudb::data::get::PaginationResponse) -> Box<dyn RenderBox>
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
                    a(href=path::index_with_pagination(*page, page_size)): (format!("{}, ", page));
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
