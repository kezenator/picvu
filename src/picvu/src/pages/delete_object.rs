use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use crate::icons::{IconSize, OutlineIcon};
use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;

#[allow(dead_code)]
pub struct DeleteObjectPage
{
}

impl DeleteObjectPage
{
    pub fn path_for(obj_id: &picvudb::data::ObjectId) -> String
    {
        format!("/delete/object/{}", obj_id.to_string())
    }
}

impl PageResources for DeleteObjectPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/delete/object/{obj_id}", web::get().to(get_delete_object))
            .route_other("/form/delete_object/{obj_id}", web::post().to(post_delete_object));
    }
}

async fn get_object(state: &web::Data<State>, object_id: &picvudb::data::ObjectId) -> Result<Option<picvudb::data::get::ObjectMetadata>, view::ErrorResponder>
{
    let query = picvudb::data::get::GetObjectsQuery::ByObjectId(object_id.clone());

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination: None,
    };

    let response = state.db.send(msg).await??;
    let mut objects = response.objects;
    let object = objects.drain(..).nth(0);

    Ok(object)
}

async fn get_delete_object(state: web::Data<State>, object_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
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
            Ok(render_delete_object(object, &req, &state.header_links))
        },
    }
}

async fn post_delete_object(state: web::Data<State>, object_id: web::Path<String>) -> Result<HttpResponse, view::ErrorResponder>
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
            let remove = object.tags.iter().map(|t| t.tag_id.clone()).collect();

            let add = vec![picvudb::data::add::Tag
            {
                name: "Trash".to_owned(),
                kind: picvudb::data::TagKind::Trash,
                rating: picvudb::data::Rating::NotRated,
                censor: picvudb::data::Censor::FamilyFriendly,
            }];

            let msg = picvudb::msgs::UpdateObjectTagsRequest
            {
                object_id,
                remove,
                add,
            };

            state.db.send(msg).await??;
            
            Ok(view::redirect("/".to_owned()))
        },
    }
}

fn render_delete_object(object: picvudb::data::get::ObjectMetadata, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let filename = object.attachment.filename.clone();

    let title = view::Title
    {
        text: object.title.clone().map(|m| m.get_display_text()).unwrap_or(filename.clone()),
        html: Raw(object.title.clone().map(|m| m.get_html()).unwrap_or(owned_html!{ : filename.clone() }.into_string().unwrap())),
    };

    let title_html = title.html.clone();

    let contents = owned_html!
    {
        form(method="POST", action=format!("/form/delete_object/{}", object.id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            h1
            {
                : OutlineIcon::AlertTriangle.render(IconSize::Size32x32);
                : " Warning!";
            }

            p
            {
                : "This will move \"";
                : title_html;
                : "\" to the trash.";
            }

            p
            {
                : "Any tags it may have had will be lost. However, the object will still exist in the trash until it is cleared.";
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
