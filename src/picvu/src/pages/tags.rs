use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Template};

use crate::icons::{IconSize, OutlineIcon};
use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::pages;
use crate::bulk;

#[allow(dead_code)]
pub struct TagPages
{
}

impl TagPages
{
    pub fn edit_path(tag_id: &picvudb::data::TagId) -> String
    {
        format!("/edit/tag/{}", tag_id.to_string())
    }

    pub fn delete_path(tag_id: &picvudb::data::TagId) -> String
    {
        format!("/delete/tag/{}", tag_id.to_string())
    }
}

impl PageResources for TagPages
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/edit/tag/{tag_id}", web::get().to(get_edit_tag))
            .route_view("/delete/tag/{tag_id}", web::get().to(get_delete_tag))
            .route_other("/form/edit_tag/{tag_id}", web::post().to(post_edit_tag))
            .route_other("/form/delete_tag/{tag_id}", web::post().to(post_delete_tag));
    }
}

async fn get_edit_tag(state: web::Data<State>, tag_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let tag_id = picvudb::data::TagId::try_new(tag_id.to_string())?;

    let tag = state.db.send(picvudb::msgs::GetTagRequest{ tag_id }).await??.tag;

    Ok(render_edit_tag(tag, &req, &state.header_links))
}

async fn get_delete_tag(state: web::Data<State>, tag_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let tag_id = picvudb::data::TagId::try_new(tag_id.to_string())?;

    let tag = state.db.send(picvudb::msgs::GetTagRequest{ tag_id: tag_id.clone() }).await??.tag;
    let num_objects = state.db.send(picvudb::msgs::GetNumObjectsRequest{ query: picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag_id.clone()} }).await??.num_objects;

    Ok(render_delete_tag(tag, num_objects, &req, &state.header_links))
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

    let rating = {
        let num_stars = form.rating.parse().map_err(|_| picvudb::ParseError::new("Invalid rating"))?;
        let rating = picvudb::data::Rating::from_num_stars(num_stars)?;
        rating
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

async fn post_delete_tag(state: web::Data<State>, tag_id: web::Path<String>) -> Result<HttpResponse, view::ErrorResponder>
{
    let tag_id = picvudb::data::TagId::try_new(tag_id.to_string())?;

    let mut bulk_queue = state.bulk_queue.lock().unwrap();

    bulk_queue.enqueue(bulk::tags::DeleteTagBulkOp::new(state.db_uri.clone(), tag_id));

    Ok(view::redirect("/".to_owned()))
}

fn render_edit_tag(tag: picvudb::data::get::TagMetadata, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let title = format!("Edit Tag {}", tag.name);

    let contents = owned_html!
    {
        script(src="/assets/picvu.js");
        script(src="/assets/edit_tag.js");

        form(id="form", method="POST", action=format!("/form/edit_tag/{}", tag.tag_id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            div(class="cmdbar cmdbar-top")
            {
                a(id="save", href="javascript:picvu.submit();", class="cmdbar-link")
                {
                    : OutlineIcon::Save.render(IconSize::Size16x16);
                    : " Save"
                }
                a(href=pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag.tag_id.clone(), }), class="cmdbar-link")
                {
                    : OutlineIcon::Cancel.render(IconSize::Size16x16);
                    : " Cancel"
                }
                div(class="cmdbar-summary")
                {
                }
            }

            label(for="name")
            {
                : "Name";
            }
            input(id="edit-name", type="text", name="name", value=tag.name);

            : pages::templates::tag_kind::render("kind", &tag.kind);
            : pages::templates::rating::render("rating", &tag.rating);
            : pages::templates::censor::render("censor", &tag.censor);
        }
    }.into_string().unwrap();

    view::html_page(req, header_links, title, OutlineIcon::Edit, &contents)
}

fn render_delete_tag(tag: picvudb::data::get::TagMetadata, num_objects: u64, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let title = format!("Delete Tag {}", tag.name);

    let contents = owned_html!
    {
        form(method="POST", action=format!("/form/delete_tag/{}", tag.tag_id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            h1
            {
                : OutlineIcon::AlertTriangle.render(IconSize::Size32x32);
                : " Warning!";
            }

            p
            {
                : format!("This will remove the tag \"{}\" from {} images, and then remove this tag from the system.", tag.name, num_objects);
            }

            p
            {
                : "This action cannot be undone!";
            }

            input(type="submit", value="Delete");
        }

    }.into_string().unwrap();

    view::html_page(req, header_links, title, OutlineIcon::Trash2, &contents)
}
