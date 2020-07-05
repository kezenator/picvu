use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use crate::icons::OutlineIcon;
use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::pages;

#[allow(dead_code)]
pub struct EditObjectPage
{
}

impl EditObjectPage
{
    pub fn path_for(obj_id: &picvudb::data::ObjectId) -> String
    {
        format!("/edit/object/{}", obj_id.to_string())
    }
}

impl PageResources for EditObjectPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/edit/object/{obj_id}", web::get().to(get_edit_object))
            .route_other("/form/edit_object/{obj_id}", web::post().to(post_edit_object));
    }
}

async fn get_object(state: &web::Data<State>, object_id: &picvudb::data::ObjectId) -> Result<Option<picvudb::data::get::ObjectMetadata>, view::ErrorResponder>
{
    let query = picvudb::data::get::GetObjectsQuery::ByObjectId(object_id.clone());
    let pagination = picvudb::data::get::PaginationRequest
    {
        offset: 0,
        page_size: 25,
    };

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination,
    };

    let response = state.db.send(msg).await??;
    let mut objects = response.objects;
    let object = objects.drain(..).nth(0);

    Ok(object)
}

async fn get_edit_object(state: web::Data<State>, object_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

    match get_object(&state, &object_id).await?
    {
        None =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Not Found"))
        },
        Some(object) =>
        {
            Ok(render_edit_object(object, &req, &state.header_links))
        },
    }
}

#[derive(Deserialize)]
struct FormEditObject
{
    activity: String,
    title: String,
    notes: String,
    rating: String,
    censor: String,
    location: String,
}

async fn post_edit_object(state: web::Data<State>, object_id: web::Path<String>, form: web::Form<FormEditObject>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

    let object = match get_object(&state, &object_id).await?
    {
        None =>
        {
            return Ok(view::err(HttpResponse::NotFound(), "Not Found"))
        },
        Some(object) =>
        {
            object
        },
    };

    let activity_time = picvudb::data::Date::from_rfc3339(&form.activity)?;
    let title = if form.title.is_empty() { None } else { Some(picvudb::data::TitleMarkdown::parse(form.title.clone())?) };
    let notes = if form.notes.is_empty() { None } else { Some(picvudb::data::NotesMarkdown::parse(form.notes.clone())?) };

    let rating =
    {
        if form.rating.is_empty()
        {
            None
        }
        else
        {
            let num_stars = form.rating.parse().map_err(|_| picvudb::ParseError::new("Invalid rating"))?;
            let rating = picvudb::data::Rating::from_num_stars(num_stars)?;
            Some(rating)
        }
    };

    let censor: picvudb::data::Censor = form.censor.parse()?;

    let location = if form.location.is_empty() { None } else { Some(form.location.parse()?) };

    if activity_time != object.activity_time
        || title != object.title
        || notes != object.notes
        || rating != object.rating
        || censor != object.censor
        || location != object.location
    {
        let msg = picvudb::msgs::UpdateObjectRequest
        {
            object_id: object_id.clone(),
            activity_time,
            title,
            notes,
            rating,
            censor,
            location,
        };

        let _response = state.db.send(msg).await??;
    }

    Ok(view::redirect(pages::object_details::ObjectDetailsPage::path_for(&object_id)))
}

fn render_edit_object(object: picvudb::data::get::ObjectMetadata, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let filename = object.attachment.filename.clone();

    let title = view::Title
    {
        text: object.title.clone().map(|m| m.get_display_text()).unwrap_or(filename.clone()),
        html: Raw(object.title.clone().map(|m| m.get_html()).unwrap_or(owned_html!{ : filename.clone() }.into_string().unwrap())),
    };

    let contents = owned_html!
    {
        form(method="POST", action=format!("/form/edit_object/{}", object.id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            table(class="details-table")
            {
                tr
                {
                    th(colspan="2")
                    {
                        : "Edit";

                        div(class="details-table-header-right")
                        {
                            input(value="Save", type="Submit");
                        }
                    }
                }

                tr
                {
                    td: "Activity";
                    td
                    {
                        input(type="text", name="activity", value=object.activity_time.to_rfc3339());
                    }
                }

                tr
                {
                    td: "Title";
                    td
                    {
                        input(type="text", name="title", value=object.title.clone().map(|m| m.get_markdown()).unwrap_or_default());
                    }
                }

                tr
                {
                    td: "Notes";
                    td
                    {
                        textarea(name="notes", rows=10, cols=60)
                        {
                            : object.notes.clone().map(|m| m.get_markdown()).unwrap_or_default();
                        }
                    }
                }

                tr
                {
                    td: "Rating";
                    td
                    {
                        select(name="rating")
                        {
                            option(value="", selected?=object.rating.is_none()) { : "Unrated" }
                            option(value="1", selected?=(object.rating == Some(picvudb::data::Rating::OneStar))) { : "1 Star" }
                            option(value="2", selected?=(object.rating == Some(picvudb::data::Rating::TwoStars))) { : "2 Stars" }
                            option(value="3", selected?=(object.rating == Some(picvudb::data::Rating::ThreeStars))) { : "3 Stars" }
                            option(value="4", selected?=(object.rating == Some(picvudb::data::Rating::FourStars))) { : "4 Stars" }
                            option(value="5", selected?=(object.rating == Some(picvudb::data::Rating::FiveStars))) { : "5 Stars" }
                        }
                    }
                }

                tr
                {
                    td: "Censor";
                    td
                    {
                        select(name="censor")
                        {
                            @for c in [picvudb::data::Censor::FamilyFriendly, picvudb::data::Censor::TastefulNudes,
                                            picvudb::data::Censor::FullNudes, picvudb::data::Censor::Explicit].iter()
                            {
                                option(
                                    value=c.to_string(),
                                    selected?=(object.censor == *c))
                                {
                                    : c.to_string()
                                }
                            }
                        }
                    }
                }

                tr
                {
                    td: "Location";
                    td
                    {
                        input(type="text", name="location", value=object.location.clone().map(|l| l.to_string()).unwrap_or_default());
                    }
                }

                tr
                {
                    th(colspan="2"): "Preview";
                }

                tr
                {
                    td(colspan="2")
                    {
                        a(href=pages::attachments::AttachmentsPage::path_attachment(&object.id, &object.attachment.hash))
                        {
                            : pages::attachments::AttachmentsPage::raw_html_for_thumbnail(&object, 512, true);
                        }
                    }
                }
            }
        }
    }.into_string().unwrap();

    view::html_page(req, header_links, title, OutlineIcon::Edit, &contents)
}
