use std::collections::BTreeMap;
use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use picvudb::msgs::GetObjectsResponse;

use crate::icons::{ColoredIcon, Icon, IconSize, OutlineIcon};
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

#[derive(Deserialize)]
pub struct LocationListViewOptionsForm
{
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub location: picvudb::data::Location,
    pub radius_meters: f64,
    pub list_type: Option<ViewObjectsListType>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct SearchListViewOptionsForm
{
    pub q: String,
    pub list_type: Option<ViewObjectsListType>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct TagListViewOptionsForm
{
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub tag_id: picvudb::data::TagId,
    pub list_type: Option<ViewObjectsListType>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct ActivityRangeListViewOptionsForm
{
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub date_range: picvudb::data::DateRange,
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
    fn base_url(query: picvudb::data::get::GetObjectsQuery) -> (String, Vec<(&'static str, String)>)
    {
        let mut params = Vec::new();

        let base_url = match &query
        {
            picvudb::data::get::GetObjectsQuery::ByActivityDesc => "/view/objects/by_activity_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByModifiedDesc => "/view/objects/by_modified_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc => "/view/objects/by_size_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::ByObjectId(obj_id) => pages::object_details::ObjectDetailsPage::path_for(&obj_id),
            picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ .. } => "/view/objects/near_location_by_activity_desc".to_owned(),
            picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ .. } => "/view/objects/search".to_owned(),
            picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ .. } => "/view/objects/by_tag".to_owned(),
            picvudb::data::get::GetObjectsQuery::ActivityDateRangeByActivityDesc{ .. } => "/view/objects/by_activity_range_desc".to_owned(),
        };

        if let picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters} = query
        {
            params.push(("location", location.to_string()));
            params.push(("radius_meters", radius_meters.to_string()));
        }
        else if let picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ search } = query
        {
            params.push(("q", search.to_literal_string()));
        }
        else if let picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id } = query
        {
            params.push(("tag_id", tag_id.to_string()));
        }
        else if let picvudb::data::get::GetObjectsQuery::ActivityDateRangeByActivityDesc{ date_range } = query
        {
            params.push(("date_range", date_range.to_string()));
        }

        (base_url, params)
    }

    fn encode(base_uri: String, params: Vec<(&'static str, String)>) -> String
    {
        let mut result = base_uri;

        if !params.is_empty()
        {
            result.push('?');

            for i in 0..params.len()
            {
                if i != 0
                {
                    result.push('&');
                }
                result.push_str(params[i].0);
                result.push('=');
                result.push_str(&params[i].1);
            }
        }

        result
    }

    pub fn path(query: picvudb::data::get::GetObjectsQuery) -> String
    {
        let (base_url, params) = Self::base_url(query);
        Self::encode(base_url, params)
    }
    
    pub fn path_with_options(query: picvudb::data::get::GetObjectsQuery, list_type: ViewObjectsListType, offset: u64, page_size: u64) -> String
    {
        let (base_url, mut params) = Self::base_url(query);

        params.push(("list_type", format!("{:?}", list_type)));
        params.push(("offset", offset.to_string()));
        params.push(("page_size", page_size.to_string()));

        Self::encode(base_url, params)
    }

    pub fn icon(query: &picvudb::data::get::GetObjectsQuery) -> Icon
    {
        match query
        {
            picvudb::data::get::GetObjectsQuery::ByActivityDesc => OutlineIcon::Calendar,
            picvudb::data::get::GetObjectsQuery::ByModifiedDesc => OutlineIcon::List,
            picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc => OutlineIcon::FilePlus,
            picvudb::data::get::GetObjectsQuery::ByObjectId(_) => OutlineIcon::Edit,
            picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ .. } => OutlineIcon::Location,
            picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ .. } => OutlineIcon::Search,
            picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ .. } => OutlineIcon::Label,
            picvudb::data::get::GetObjectsQuery::ActivityDateRangeByActivityDesc{ .. } => OutlineIcon::Calendar,
        }.into()
    }
}

impl PageResources for ObjectListingPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/view/objects/by_activity_desc", "Calendar", OutlineIcon::Calendar, 0)
            .add_header_link("/view/objects/by_modified_desc", "Recently Modified", OutlineIcon::List, 1)
            .add_header_link("/view/objects/by_size_desc", "Largest Attachments", OutlineIcon::FilePlus, 2)
            .route_view("/view/objects/by_modified_desc", web::get().to(objects_by_modified_desc))
            .route_view("/view/objects/by_activity_desc", web::get().to(objects_by_activity_desc))
            .route_view("/view/objects/by_size_desc", web::get().to(objects_by_size_desc))
            .route_view("/view/objects/near_location_by_activity_desc", web::get().to(objects_near_location_by_activity_desc))
            .route_view("/view/objects/search", web::get().to(objects_search))
            .route_view("/view/objects/by_tag", web::get().to(objects_by_tag))
            .route_view("/view/objects/by_activity_range_desc", web::get().to(objects_by_activity_range_desc));
    }
}

pub async fn object_query(state: web::Data<State>, options: &ListViewOptionsForm, query: picvudb::data::get::GetObjectsQuery, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    // TODO - this should be put into a middle-ware
    // that wraps all user-interface pages
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        if let Some(_progress) = bulk_queue.get_current_progress()
        {
            return Ok(view::redirect(pages::bulk::BulkPage::progress_path()));
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
        pagination: Some(pagination.clone()),
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

    let tags = if let picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{search} = &response.query
    {
        let tags_msg = picvudb::msgs::SearchTagsRequest{ search: search.clone() };

        state.db.send(tags_msg).await??.tags
    }
    else
    {
        Vec::new()
    };

    let search_tag = if let picvudb::data::get::GetObjectsQuery::TagByActivityDesc{tag_id} = &response.query
    {
        let get_tag_msg = picvudb::msgs::GetTagRequest{ tag_id: tag_id.clone() };

        Some(state.db.send(get_tag_msg).await??.tag)
    }
    else
    {
        None
    };

    Ok(render_object_listing(
        response,
        tags,
        search_tag,
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

async fn objects_near_location_by_activity_desc(state: web::Data<State>, query: web::Query<LocationListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let options = ListViewOptionsForm
    {
        list_type: query.list_type,
        offset: query.offset,
        page_size: query.page_size,
    };

    let query = picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc
    {
        location: query.location.clone(),
        radius_meters: query.radius_meters,
    };

    object_query(state, &options, query, req).await
}

async fn objects_search(state: web::Data<State>, query: web::Query<SearchListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let options = ListViewOptionsForm
    {
        list_type: query.list_type,
        offset: query.offset,
        page_size: query.page_size,
    };

    let query = picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc
    {
        search: picvudb::data::get::SearchString::FullSearch(query.q.clone()),
    };

    object_query(state, &options, query, req).await
}

async fn objects_by_tag(state: web::Data<State>, query: web::Query<TagListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let options = ListViewOptionsForm
    {
        list_type: query.list_type,
        offset: query.offset,
        page_size: query.page_size,
    };

    let query = picvudb::data::get::GetObjectsQuery::TagByActivityDesc
    {
        tag_id: query.tag_id.clone(),
    };

    object_query(state, &options, query, req).await
}

async fn objects_by_activity_range_desc(state: web::Data<State>, query: web::Query<ActivityRangeListViewOptionsForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let options = ListViewOptionsForm
    {
        list_type: query.list_type,
        offset: query.offset,
        page_size: query.page_size,
    };

    let query = picvudb::data::get::GetObjectsQuery::ActivityDateRangeByActivityDesc
    {
        date_range: query.date_range.clone(),
    };

    object_query(state, &options, query, req).await
}

pub fn render_object_listing(resp: GetObjectsResponse, tags: Vec<picvudb::data::get::TagMetadata>, search_tag: Option<picvudb::data::get::TagMetadata>, list_type: ViewObjectsListType, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    match list_type
    {
        ViewObjectsListType::DetailsTable => render_objects_details(resp, tags, search_tag, req, header_links),
        ViewObjectsListType::ThumbnailsGrid => render_objects_thumbnails(resp, tags, search_tag, req, header_links),
    }
}

pub fn render_objects_thumbnails(resp: GetObjectsResponse, tags: Vec<picvudb::data::get::TagMetadata>, search_tag: Option<picvudb::data::get::TagMetadata>, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let (title, icon) = get_title_and_icon(&resp.query, &search_tag);

    let mut cur_heading = String::new();

    let contents = owned_html!{

        : get_query_commands(&resp.query);

        : (pagination(resp.query.clone(), tags.len(), ViewObjectsListType::ThumbnailsGrid, resp.pagination_response.clone(), true));

        div(class="object-listing")
        {
            @if resp.pagination_response.offset == 0
            {
                @if !tags.is_empty()
                {
                    h2(class="object-listing-group")
                    {
                        : "Tags";
                    }

                    div(class="object-listing-tags")
                    {
                        @for tag in tags.iter()
                        {
                            a(href=pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag.tag_id.clone() }),
                                class="tag")
                            {
                                : pages::templates::tags::render_existing(tag);
                            }
                        }
                    }
                }
            }

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

                        div(class="object-listing-tags")
                        {
                            @for tag in get_tags_for_objects_with_heading(&cur_heading, &resp.objects, &resp.query)
                            {
                                a(href=pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag.tag_id.clone() }),
                                    class="tag")
                                {
                                    : pages::templates::tags::render_existing(&tag);
                                }
                            }
                        }
                    }
                }

                : pages::templates::thumbnails::render(
                    object,
                    pages::object_details::ObjectDetailsPage::path_for(&object.id),
                    false);
            }
        }

        : (pagination(resp.query.clone(), tags.len(), ViewObjectsListType::ThumbnailsGrid, resp.pagination_response.clone(), false));

    }.into_string().unwrap();

    view::html_page(req, header_links, &title, icon, &contents)
}

pub fn render_objects_details(resp: GetObjectsResponse, tags: Vec<picvudb::data::get::TagMetadata>, search_tag: Option<picvudb::data::get::TagMetadata>, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let now = picvudb::data::Date::now();

    let (title, icon) = get_title_and_icon(&resp.query, &search_tag);

    let contents = owned_html!{

        : get_query_commands(&resp.query);

        : (pagination(resp.query.clone(), tags.len(), ViewObjectsListType::DetailsTable, resp.pagination_response.clone(), true));

        table(class="details-table")
        {
            tr
            {
                th: "Title";
                th: "Activity";
                th: "Info";
                th: "Size";
                th: "Mime";
                th: "Dimensions";
                th: "Duration";
                th: "Location";
            }

            @if resp.pagination_response.offset == 0
            {
                @for tag in tags.iter()
                {
                    tr
                    {
                        td
                        {
                            : "Tag ";
                            a(href=pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag.tag_id.clone() }))
                            {
                                : (match tag.kind
                                    {
                                        picvudb::data::TagKind::Activity => OutlineIcon::Sun,
                                        picvudb::data::TagKind::Event => OutlineIcon::Calendar,
                                        picvudb::data::TagKind::Label => OutlineIcon::Label,
                                        picvudb::data::TagKind::List => OutlineIcon::List,
                                        picvudb::data::TagKind::Location => OutlineIcon::Location,
                                        picvudb::data::TagKind::Person => OutlineIcon::User,
                                        picvudb::data::TagKind::Trash => OutlineIcon::Trash2,
                                        picvudb::data::TagKind::Unsorted => OutlineIcon::PatchQuestion,
                                    }).render(IconSize::Size16x16);
                                : &tag.name
                            }
                        }
                        td {}
                        td
                        {
                            @if tag.rating != picvudb::data::Rating::NotRated
                            {
                                : ColoredIcon::Star.render(IconSize::Size16x16);
                            }
    
                            : (match tag.censor
                                {
                                    picvudb::data::Censor::FamilyFriendly => Raw(String::new()),
                                    picvudb::data::Censor::TastefulNudes => ColoredIcon::Peach.render(IconSize::Size16x16),
                                    picvudb::data::Censor::FullNudes => ColoredIcon::Eggplant.render(IconSize::Size16x16),
                                    picvudb::data::Censor::Explicit => ColoredIcon::EvilGrin.render(IconSize::Size16x16),
                                });
                        }
                        td {}
                        td {}
                        td {}
                        td {}
                        td {}
                    }
                }
            }

            @for object in resp.objects.iter()
            {
                tr
                {
                    td
                    {
                        a(href=pages::object_details::ObjectDetailsPage::path_for(&object.id))
                        {
                            @if let Some(title) = &object.title
                            {
                                : Raw(title.get_html())
                            }
                            else
                            {
                                : &object.attachment.filename
                            }
                        }
                    }

                    td: format::date_to_str(&object.activity_time, &now);

                    td
                    {
                        @if object.notes.is_some()
                        {
                            : ColoredIcon::Memo.render(IconSize::Size16x16);
                        }
                        @if object.rating != picvudb::data::Rating::NotRated
                        {
                            : ColoredIcon::Star.render(IconSize::Size16x16);
                        }

                        : (match object.censor
                            {
                                picvudb::data::Censor::FamilyFriendly => Raw(String::new()),
                                picvudb::data::Censor::TastefulNudes => ColoredIcon::Peach.render(IconSize::Size16x16),
                                picvudb::data::Censor::FullNudes => ColoredIcon::Eggplant.render(IconSize::Size16x16),
                                picvudb::data::Censor::Explicit => ColoredIcon::EvilGrin.render(IconSize::Size16x16),
                            });
                    }

                    td: format::bytes_to_string(object.attachment.size);
                    td: object.attachment.mime.to_string();
                    td: object.attachment.dimensions.clone().map(|d| d.to_string()).unwrap_or_default();
                    td: object.attachment.duration.clone().map(|d| d.to_string()).unwrap_or_default();
                    td: object.location.clone().map(|l| l.to_string()).unwrap_or_default();
                }
            }
        }

        : (pagination(resp.query.clone(), tags.len(), ViewObjectsListType::DetailsTable, resp.pagination_response.clone(), false));

    }.into_string().unwrap();

    view::html_page(req, header_links, &title, icon, &contents)
}

fn get_title_and_icon(query: &picvudb::data::get::GetObjectsQuery, search_tag: &Option<picvudb::data::get::TagMetadata>) -> (String, Icon)
{
    let mut title = format::query_to_string(query);
    let mut icon = ObjectListingPage::icon(query);

    if let Some(tag) = search_tag
    {
        title = format!("Tagged: {}", tag.name);
        icon = match tag.kind
        {
            picvudb::data::TagKind::Activity => OutlineIcon::Sun,
            picvudb::data::TagKind::Event => OutlineIcon::Calendar,
            picvudb::data::TagKind::Label => OutlineIcon::Label,
            picvudb::data::TagKind::List => OutlineIcon::List,
            picvudb::data::TagKind::Location => OutlineIcon::Location,
            picvudb::data::TagKind::Person => OutlineIcon::User,
            picvudb::data::TagKind::Trash => OutlineIcon::Trash2,
            picvudb::data::TagKind::Unsorted => OutlineIcon::PatchQuestion,
        }.into();
    }

    (title, icon)
}

fn get_query_commands(query: &picvudb::data::get::GetObjectsQuery) -> Raw<String>
{
    if let picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id } = query
    {
        return Raw(owned_html!
        {
            div(class="cmdbar cmdbar-top")
            {
                a(href=pages::tags::TagPages::edit_path(tag_id), class="cmdbar-link")
                {
                    : OutlineIcon::Edit.render(IconSize::Size16x16);
                    : " Edit Tag"
                }
                a(href=pages::tags::TagPages::delete_path(tag_id), class="cmdbar-link")
                {
                    : OutlineIcon::Trash2.render(IconSize::Size16x16);
                    : " Delete Tag"
                }
                div(class="cmdbar-summary")
                {
                }
            }
        }.into_string().unwrap());
    }

    Raw(String::new())
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
        picvudb::data::get::GetObjectsQuery::ByActivityDesc
            | picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ .. }
            | picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ .. }
            | picvudb::data::get::GetObjectsQuery::TagByActivityDesc { .. }
            | picvudb::data::get::GetObjectsQuery::ActivityDateRangeByActivityDesc{ .. } =>
        {
            format::date_to_date_only_string(&object.activity_time)
        },
        picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc =>
        {
            format::bytes_to_group_header(object.attachment.size)
        },

    }
}

fn get_tags_for_objects_with_heading(heading: &str, objects: &Vec<picvudb::data::get::ObjectMetadata>, query: &picvudb::data::get::GetObjectsQuery) -> Vec<picvudb::data::get::TagMetadata>
{
    let mut tags = BTreeMap::new();

    for obj in objects.iter()
    {
        if heading == get_heading(obj, query)
        {
            for tag in obj.tags.iter()
            {
                tags.insert(tag.name.clone(), tag.clone());
            }
        }
    }

    tags.into_iter().map(|(_name, tag)| tag).collect()
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

fn pagination(query: picvudb::data::get::GetObjectsQuery, num_tags: usize, list_type: ViewObjectsListType, response: picvudb::data::get::PaginationResponse, top: bool) -> Raw<String>
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
        div(class=(if top { "cmdbar cmdbar-top" } else { "cmdbar cmdbar-bottom" }))
        {
            @for page in pages.iter()
            {
                @if should_print_page(*page, cur_page, last_page)
                {
                    : ({ done_elipsis = false; ""});

                    a(href=ObjectListingPage::path_with_options(query.clone(), list_type, (*page - 1) * page_size, page_size),
                        class=(if cur_page == *page { "cmdbar-link cmdbar-selected" } else { "cmdbar-link" }))
                    {
                        : (format!("{}, ", page));
                    }
                }
                else
                {
                    @if !done_elipsis
                    {
                        div(class="cmdbar-elipsis")
                        {
                            : ({ done_elipsis = true; "..." });
                        }
                    }
                }
            }

            div(class="cmdbar-summary")
            {
                : (format!("Total: {} objects", total));

                @if num_tags != 0
                {
                    : format!(", {} tags", num_tags);
                }

                : " ";
            }

            a(href=ObjectListingPage::path_with_options(query.clone(), ViewObjectsListType::ThumbnailsGrid, this_page_offset, page_size),
                class=(if list_type == ViewObjectsListType::ThumbnailsGrid { "cmdbar-link cmdbar-selected" } else { "cmdbar-link" }))
            {
                : OutlineIcon::Image.render(IconSize::Size16x16);
                : " Thumbnails ";
            }

            a(href=ObjectListingPage::path_with_options(query.clone(), ViewObjectsListType::DetailsTable, this_page_offset, page_size),
                class=(if list_type == ViewObjectsListType::DetailsTable { "cmdbar-link cmdbar-selected" } else { "cmdbar-link" }))
            {
                : OutlineIcon::List.render(IconSize::Size16x16);
                : " Details ";
            }
        }
    }.into_string().unwrap();

    Raw(result)
}
