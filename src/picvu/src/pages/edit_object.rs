use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use crate::icons::{IconSize, OutlineIcon};
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
            .route_other("/edit/find_tags", web::get().to(get_find_tags))
            .route_other("/form/edit_object/{obj_id}", web::post().to(post_edit_object));
    }
}

async fn get_object(state: &web::Data<State>, object_id: &picvudb::data::ObjectId) -> Result<picvudb::msgs::GetObjectsForEditResponse, view::ErrorResponder>
{
    let msg = picvudb::msgs::GetObjectsForEditRequest
    {
        object_id: object_id.clone(),
    };

    let response = state.db.send(msg).await??;

    Ok(response)
}

async fn get_edit_object(state: web::Data<State>, object_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

    let response = get_object(&state, &object_id).await?;

    if response.object.is_none()
    {
        Ok(view::err(HttpResponse::NotFound(), "Not Found"))
    }
    else
    {
        Ok(render_edit_object(response.object.unwrap(), response.all_objects_on_same_date, &req, &state.header_links))
    }
}

#[derive(Deserialize)]
struct FormEditObject
{
    next: String,
    activity: String,
    title: String,
    notes: String,
    rating: String,
    censor: String,
    location: String,
    remove_tag_id: String,
    add_tag_name: String,
}

async fn post_edit_object(state: web::Data<State>, object_id: web::Path<String>, form: web::Form<FormEditObject>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;
    let next_object_id = picvudb::data::ObjectId::try_new(form.next.clone())?;

    let object = match get_object(&state, &object_id).await?.object
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

    let location = if form.location.is_empty()
    {
        None
    }
    else
    {
        let mut new = form.location.parse::<picvudb::data::Location>()?;

        if let Some(cur) = object.location.clone()
        {
            if ((new.latitude - cur.latitude).abs() < 1e-9)
                && ((new.latitude - cur.latitude).abs() < 1e-9)
            {
                // The new location is withing 1e-9 of the current location -
                // just keep the current location, so we keep the same source
                // and altitude, which we don't support parsing from a string.

                new = cur;
            }
        }

        Some(new)
    };

    let mut details = None;
    let mut tags = None;

    if activity_time != object.activity_time
        || title != object.title
        || notes != object.notes
        || rating != object.rating
        || censor != object.censor
        || location != object.location
    {
        
        details = Some(picvudb::msgs::UpdateObjectRequest
        {
            object_id: object_id.clone(),
            activity_time,
            title,
            notes,
            rating,
            censor,
            location,
        });
    }

    if !form.remove_tag_id.is_empty()
        || !form.add_tag_name.is_empty()
    {
        let mut remove = Vec::new();
        let mut add = Vec::new();

        if !form.remove_tag_id.is_empty()
        {
            remove.push(form.remove_tag_id.parse::<picvudb::data::TagId>()?);
        }
        
        if !form.add_tag_name.is_empty()
        {
            add.push(picvudb::data::add::Tag {
                name: form.add_tag_name.clone(),
                kind: picvudb::data::TagKind::Label,
                rating: None,
                censor: picvudb::data::Censor::FamilyFriendly,
            });
        }

        if let Some(unsorted_tag_id) = object.tags.iter()
            .filter(|tag| tag.name == "Unsorted")
            .map(|tag| tag.tag_id.clone())
            .next()
        {
            // This object still has the unsorted tag - since we've adjusted
            // some tags, we should mark this as now sorted

            if remove.iter().position(|tag_id| *tag_id == unsorted_tag_id).is_none()
            {
                remove.push(unsorted_tag_id);
            }
        }

        tags = Some(picvudb::msgs::UpdateObjectTagsRequest{
            object_id: object_id.clone(),
            remove,
            add,
        });
    }

    let msg = picvudb::msgs::EditObjectRequest
    {
        details,
        tags,
    };

    let _ = state.db.send(msg).await??;

    Ok(view::redirect(pages::edit_object::EditObjectPage::path_for(&next_object_id)))
}

#[derive(Deserialize)]
struct FormFindTags
{
    name: String,
}

async fn get_find_tags(state: web::Data<State>, form: web::Query<FormFindTags>) -> Result<HttpResponse, view::ErrorResponder>
{
    if form.name.is_empty()
    {
        return Ok(view::html_fragment(String::new()));
    }

    let msg = picvudb::msgs::SearchTagsRequest { search: picvudb::data::get::SearchString::Suggestion(form.name.clone()) };

    let response = state.db.send(msg).await??;

    let fragment = owned_html!
    {
        @if response.tags.is_empty()
        {
            : "TODO - full add form";
        }
        else
        {
            h2
            {
                : "Suggestions";
            }

            @for tag in response.tags
            {
                a(href=format!(
                        "javascript:submit_funcs.add_tag('{}');",
                        urlencoding::encode(&tag.name)),
                    class="add-tag")
                {
                    : OutlineIcon::Import.render(IconSize::Size16x16);
                    : " Add ";
                    : pages::templates::tags::render(&tag);
                }
            }
        }
    }.into_string().unwrap();

    Ok(view::html_fragment(fragment))
}

fn render_edit_object(object: picvudb::data::get::ObjectMetadata, all_objs_on_date: Vec<picvudb::data::get::ObjectMetadata>, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let filename = object.attachment.filename.clone();

    let title = view::Title
    {
        text: format!("Edit: {}", object.title.clone().map(|m| m.get_display_text()).unwrap_or(filename.clone())),
        html: Raw(format!("Edit: {}", object.title.clone().map(|m| m.get_html()).unwrap_or(owned_html!{ : filename.clone() }.into_string().unwrap()))),
    };

    let mut tags_on_same_day : Vec<picvudb::data::get::TagMetadata> = all_objs_on_date.iter()
        .map(|obj| obj.tags.clone())
        .flatten()
        .map(|tag| (tag.tag_id.clone(), tag))
        .collect::<std::collections::HashMap<_, _>>()
        .values()
        .filter(|tag| object.tags.iter().position(|otag| otag.tag_id == tag.tag_id).is_none())
        .filter(|tag| tag.name != "Unsorted")
        .filter(|tag| tag.name != "Trash")
        .cloned()
        .collect();

    tags_on_same_day.sort_by(|a, b| a.name.cmp(&b.name));

    let index_of_this_obj = all_objs_on_date.iter()
        .position(|o| o.id == object.id)
        .unwrap_or(0);

    let contents = owned_html!
    {
        script(defer, src="/assets/edit.js");

        form(id="form", method="POST", action=format!("/form/edit_object/{}", object.id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            div(class="cmdbar cmdbar-top")
            {
                a(href="javascript:submit_funcs.submit();", class="cmdbar-link")
                {
                    : OutlineIcon::Save.render(IconSize::Size16x16);
                    : " Save"
                }
                a(href=pages::object_details::ObjectDetailsPage::path_for(&object.id), class="cmdbar-link")
                {
                    : OutlineIcon::Cancel.render(IconSize::Size16x16);
                    : " Cancel"
                }
                div(class="cmdbar-summary")
                {
                }
            }

            input(id="hidden-next-object-id", type="hidden", name="next", value=object.id.to_string());

            div(class="object-listing")
            {
                @for (index, other_obj) in all_objs_on_date.iter().enumerate()
                {
                    @if ((index + 5) >= index_of_this_obj)
                        && ((index_of_this_obj + 5) >= index)
                    {
                        : pages::templates::thumbnails::render(
                            other_obj,
                            format!("javascript:submit_funcs.move_to('{}')", other_obj.id.to_string()),
                            other_obj.id == object.id);
                    }
                }
            }
    
            table(class="details-table")
            {
                tr
                {
                    th(colspan="2")
                    {
                        : "Edit";
                    }
                }

                tr
                {
                    td: "Activity";
                    td
                    {
                        input(id="edit-activity", type="text", name="activity", value=object.activity_time.to_rfc3339());
                    }
                }

                tr
                {
                    td: "Title";
                    td
                    {
                        input(id="edit-title", type="text", name="title", value=object.title.clone().map(|m| m.get_markdown()).unwrap_or_default());
                    }
                }

                tr
                {
                    td: "Notes";
                    td
                    {
                        textarea(id="edit-notes", name="notes", rows=10, cols=60)
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
                        : pages::templates::rating::render(&object.rating);
                    }
                }

                tr
                {
                    td: "Censor";
                    td
                    {
                        : pages::templates::censor::render(&object.censor);
                    }
                }

                tr
                {
                    td: "Location";
                    td
                    {
                        input(id="edit-location", type="text", name="location", value=object.location.clone().map(|l| l.to_string()).unwrap_or_default());
                    }
                }

                tr
                {
                    th(colspan="2"): "Tags";
                }

                tr
                {
                    td(colspan="2")
                    {
                        input(id="hidden-remove-tag-id", type="hidden", name="remove_tag_id", value="");

                        @if !object.tags.is_empty()
                        {
                            h2 { : "Current Tags" }

                            @for tag in object.tags.iter()
                            {
                                a(href=format!("javascript:submit_funcs.delete_tag('{}');", tag.tag_id.to_string()),
                                    class="delete-tag")
                                {
                                    : OutlineIcon::Trash2.render(IconSize::Size16x16);
                                    : " Remove ";
                                    : pages::templates::tags::render(tag);
                                }
                            }
                        }

                        @if !tags_on_same_day.is_empty()
                        {
                            h2 { : "On Same Day" }

                            @for tag in tags_on_same_day.iter()
                            {
                                // Needs to be URI encoded as the web-browswer decodes the URI
                                // into the correct string.

                                a(href=format!(
                                        "javascript:submit_funcs.add_tag('{}');",
                                        urlencoding::encode(&&tag.name)),
                                    class="add-tag")
                                {
                                    : OutlineIcon::Import.render(IconSize::Size16x16);
                                    : " Add ";
                                    : pages::templates::tags::render(tag);
                                }
                            }
                        }

                        h2 { : "Add New Tag" }

                        input(id="edit-add-tag-name", type="text", name="add_tag_name", value="", autocomplete="off");

                        div(id="add-tags-search-div")
                        {
                        }
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
