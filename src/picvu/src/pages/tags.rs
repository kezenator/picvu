use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Template};

use crate::icons::OutlineIcon;
use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::pages;

#[allow(dead_code)]
pub struct EditTagPage
{
}

impl EditTagPage
{
    pub fn edit_path(tag_id: &picvudb::data::TagId) -> String
    {
        format!("/edit/tag/{}", tag_id.to_string())
    }
}

impl PageResources for EditTagPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/edit/tag/{tag_id}", web::get().to(get_edit_tag))
            .route_other("/form/edit_tag/{tag_id}", web::post().to(post_edit_tag));
    }
}

async fn get_edit_tag(state: web::Data<State>, tag_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let tag_id = picvudb::data::TagId::try_new(tag_id.to_string())?;

    let tag = state.db.send(picvudb::msgs::GetTagRequest{ tag_id }).await??.tag;

    Ok(render_edit_tag(tag, &req, &state.header_links))
}

#[derive(Deserialize)]
struct FormEditTag
{
    name: String,
    rating: String,
    censor: String,
    kind: String,
}

async fn post_edit_tag(state: web::Data<State>, tag_id: web::Path<String>, form: web::Form<FormEditTag>) -> Result<HttpResponse, view::ErrorResponder>
{
    let tag_id = picvudb::data::TagId::try_new(tag_id.to_string())?;

    let tag = state.db.send(picvudb::msgs::GetTagRequest{ tag_id: tag_id.clone() }).await??.tag;

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

    let kind: picvudb::data::TagKind = form.kind.parse()?;

    if form.name != tag.name
        || rating != tag.rating
        || censor != tag.censor
        || kind != tag.kind
    {
        let msg = picvudb::msgs::UpdateTagRequest
        {
            tag_id: tag_id.clone(),
            name: form.name.clone(),
            rating,
            censor,
            kind,
        };

        let _response = state.db.send(msg).await??;
    }

    Ok(view::redirect(pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id })))
}

fn render_edit_tag(tag: picvudb::data::get::TagMetadata, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let title = format!("Edit Tag {}", tag.name);

    let contents = owned_html!
    {
        form(method="POST", action=format!("/form/edit_tag/{}", tag.tag_id.to_string()), enctype="application/x-www-form-urlencoded")
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
                    td: "Name";
                    td
                    {
                        input(type="text", name="name", value=tag.name);
                    }
                }

                tr
                {
                    td: "Rating";
                    td
                    {
                        select(name="rating")
                        {
                            option(value="", selected?=tag.rating.is_none()) { : "Unrated" }
                            option(value="1", selected?=(tag.rating == Some(picvudb::data::Rating::OneStar))) { : "1 Star" }
                            option(value="2", selected?=(tag.rating == Some(picvudb::data::Rating::TwoStars))) { : "2 Stars" }
                            option(value="3", selected?=(tag.rating == Some(picvudb::data::Rating::ThreeStars))) { : "3 Stars" }
                            option(value="4", selected?=(tag.rating == Some(picvudb::data::Rating::FourStars))) { : "4 Stars" }
                            option(value="5", selected?=(tag.rating == Some(picvudb::data::Rating::FiveStars))) { : "5 Stars" }
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
                                    selected?=(tag.censor == *c))
                                {
                                    : c.to_string()
                                }
                            }
                        }
                    }
                }

                tr
                {
                    td: "Icon";
                    td
                    {
                        select(name="kind")
                        {
                            @for k in picvudb::data::TagKind::values()
                            {
                                option(
                                    value=k.to_string(),
                                    selected?=(tag.kind == k))
                                {
                                    : k.to_string()
                                }
                            }
                        }
                    }
                }
            }
        }
    }.into_string().unwrap();

    view::html_page(req, header_links, title, OutlineIcon::Edit, &contents)
}
