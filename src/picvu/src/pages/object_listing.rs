use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use picvudb::msgs::GetObjectsResponse;

use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::format;
use crate::pages;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize)]
pub enum ViewObjectsListType
{
    ThumbnailsGrid,
    DetailsTable,
}

#[derive(Deserialize)]
pub struct ListViewOptionsForm
{
    pub list_type: Option<ViewObjectsListType>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

#[allow(dead_code)]
pub struct ObjectListingPage
{
}

impl ObjectListingPage
{
    pub fn path(query: picvudb::data::get::GetObjectsQuery) -> String
    {
        match query
        {
            picvudb::data::get::GetObjectsQuery::ByActivityDesc => "/view/objects/by_activity_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByModifiedDesc => "/view/objects/by_modified_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc => "/view/objects/by_size_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByObjectId(obj_id) => pages::object_details::ObjectDetailsPage::path_for(&obj_id),
        }
    }
    
    pub fn path_with_options(query: picvudb::data::get::GetObjectsQuery, list_type: ViewObjectsListType, offset: u64, page_size: u64) -> String
    {
        format!("{}?list_type={:?}&offset={}&page_size={}", Self::path(query), list_type, offset, page_size)
    }
}

impl PageResources for ObjectListingPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/view/objects/by_activity_desc", "Calendar", 0)
            .add_header_link("/view/objects/by_modified_desc", "Recently Modified", 1)
            .add_header_link("/view/objects/by_size_desc", "Largest Attachments", 2)
            .route_view("/view/objects/by_modified_desc", web::get().to(objects_by_modified_desc))
            .route_view("/view/objects/by_activity_desc", web::get().to(objects_by_activity_desc))
            .route_view("/view/objects/by_size_desc", web::get().to(objects_by_size_desc));
    }
}

async fn object_query(state: web::Data<State>, options: &ListViewOptionsForm, query: picvudb::data::get::GetObjectsQuery, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    // TODO - this should be put into a middle-ware
    // that wraps all user-interface pages
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        if let Some(_progress) = bulk_queue.get_current_progress()
        {
            return Ok(view::redirect(pages::bulk::BulkPage::path()));
        }
    }

    let pagination = picvudb::data::get::PaginationRequest
    {
        offset: options.offset.unwrap_or(0),
        page_size: options.page_size.unwrap_or(25),
    };

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination: pagination.clone(),
    };

    let response = state.db.send(msg).await??;

    if response.pagination_response.offset != pagination.offset
        || response.pagination_response.page_size != pagination.page_size
    {
        return Ok(view::redirect(ObjectListingPage::path_with_options(
            response.query,
            options.list_type.unwrap_or(ViewObjectsListType::ThumbnailsGrid),
            response.pagination_response.offset,
            response.pagination_response.page_size)));
    }

    Ok(render_object_listing(
        response,
        options.list_type.unwrap_or(ViewObjectsListType::ThumbnailsGrid),
        &req,
        &state.header_links))
}

async fn objects_by_activity_desc(state: web::Data<State>, options: web::Query<ListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByActivityDesc, req).await
}

async fn objects_by_modified_desc(state: web::Data<State>, options: web::Query<ListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByModifiedDesc, req).await
}

async fn objects_by_size_desc(state: web::Data<State>, options: web::Query<ListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc, req).await
}

pub fn render_object_listing(resp: GetObjectsResponse, list_type: ViewObjectsListType, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    match list_type
    {
        ViewObjectsListType::DetailsTable => render_objects_details(resp, req, header_links),
        ViewObjectsListType::ThumbnailsGrid => render_objects_thumbnails(resp, req, header_links),
    }
}

pub fn render_objects_thumbnails(resp: GetObjectsResponse, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let title = format::query_to_string(&resp.query);

    let mut cur_heading = String::new();

    let contents = owned_html!{

        : (pagination(resp.query.clone(), ViewObjectsListType::ThumbnailsGrid, resp.pagination_response.clone()));

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
                        a(href=pages::object_details::ObjectDetailsPage::path_for(&object.id))
                        {
                            : pages::attachments::AttachmentsPage::raw_html_for_thumbnail(&object, 128, false);
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
        form(method="POST", action=pages::add_object::AddObjectPage::post_path(), enctype="multipart/form-data")
        {
            input(type="file", name="file", accept="image/*,video/*");
            input(type="submit");
        }
        h1: "Bulk Import";
        form(method="POST", action=pages::bulk::BulkPage::bulk_import_path(), enctype="application/x-www-form-urlencoded")
        {
            input(type="text", name="folder");
            input(type="submit");
        }
    }.into_string().unwrap();

    view::html_page(req, header_links, &title, &contents)
}

pub fn render_objects_details(resp: GetObjectsResponse, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let now = picvudb::data::Date::now();

    let title = format::query_to_string(&resp.query);

    let contents = owned_html!{

        : (pagination(resp.query.clone(), ViewObjectsListType::DetailsTable, resp.pagination_response.clone()));

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
                        a(href=pages::object_details::ObjectDetailsPage::path_for(&object.id))
                        {
                            : object.id.to_string();
                        }
                    }

                    td: object.title.clone().unwrap_or_default();
                    td: format::date_to_str(&object.activity_time, &now);
                    td: format::bytes_to_string(object.attachment.size);
                    td: object.attachment.mime.to_string();
                }
            }
        }

    }.into_string().unwrap();

    view::html_page(req, header_links, &title, &contents)
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
            format::bytes_to_group_header(object.attachment.size)
        },
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

fn pagination(query: picvudb::data::get::GetObjectsQuery, list_type: ViewObjectsListType, response: picvudb::data::get::PaginationResponse) -> Raw<String>
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
                        a(href=ObjectListingPage::path_with_options(query.clone(), list_type, (*page - 1) * page_size, page_size))
                        {
                            @if cur_page == *page
                            {
                                : (format!("[[ {} ]], ", page));
                            }
                            else
                            {
                                : (format!("{}, ", page));
                            }
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

                a(href=ObjectListingPage::path_with_options(query.clone(), ViewObjectsListType::ThumbnailsGrid, this_page_offset, page_size))
                {
                    @if list_type == ViewObjectsListType::ThumbnailsGrid
                    {
                        : " [[ Thumbnails ]] ";
                    }
                    else
                    {
                        : " Thumbnails ";
                    }
                }

                a(href=ObjectListingPage::path_with_options(query.clone(), ViewObjectsListType::DetailsTable, this_page_offset, page_size))
                {
                    @if list_type == ViewObjectsListType::DetailsTable
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
