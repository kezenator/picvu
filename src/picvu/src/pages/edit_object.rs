use std::collections::HashSet;
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
        Ok(render_edit_object(&state, response.object.unwrap(), response.all_objects_on_same_date, &req, &state.header_links))
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
    #[allow(unused)]
    search_tag_name: String,
    add_tag_name: String,
    add_tag_kind: String,
    add_tag_rating: String,
    add_tag_censor: String,
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

    let rating = {
        let num_stars = form.rating.parse().map_err(|_| picvudb::ParseError::new("Invalid rating"))?;
        let rating = picvudb::data::Rating::from_num_stars(num_stars)?;
        rating
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
            let removed_tag_id = form.remove_tag_id.parse::<picvudb::data::TagId>()?;

            if let Some(removed_tag) = object.tags.iter().filter(|t| t.tag_id == removed_tag_id).next()
            {
                let mut recent_tags = state.recent_tags.lock().unwrap();

                recent_tags.add_existing(removed_tag);
            }

            remove.push(removed_tag_id);
        }
        
        if !form.add_tag_name.is_empty()
        {
            let new_tag = picvudb::data::add::Tag {
                name: form.add_tag_name.clone(),
                kind: form.add_tag_kind.parse()?,
                rating: picvudb::data::Rating::from_num_stars(form.add_tag_rating.parse().unwrap_or(-1))?,
                censor: form.add_tag_censor.parse()?,
            };

            {
                let mut recent_tags = state.recent_tags.lock().unwrap();

                recent_tags.add_new(&new_tag);
            }

            add.push(new_tag);
        }

        let unsorted = picvudb::data::TagKind::system_name_unsorted();

        if let Some(unsorted_tag_id) = object.tags.iter()
            .filter(|tag| tag.name == unsorted)
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
    object_id: String,
    name: String,
}

async fn get_find_tags(state: web::Data<State>, form: web::Query<FormFindTags>) -> Result<HttpResponse, view::ErrorResponder>
{
    if form.name.trim().is_empty()
    {
        return Ok(view::html_fragment(String::new()));
    }

    let object_id = picvudb::data::ObjectId::try_new(form.object_id.clone())?;

    let msg = picvudb::msgs::SearchTagsRequest { search: picvudb::data::get::SearchString::Suggestion(form.name.clone()) };

    let response = state.db.send(msg).await??;

    let get_obj_msg = picvudb::msgs::GetObjectsRequest
    {
        query: picvudb::data::get::GetObjectsQuery::ByObjectId(object_id),
        pagination: None,
    };

    let obj_response = state.db.send(get_obj_msg).await??;

    let object = obj_response.objects.into_iter().next().unwrap();

    let mut tags = response.tags;

    tags.sort_by(|a, b| picvudb::stem::cmp(&a.name, &b.name));

    let found_matching = tags.iter()
        .filter(|t| picvudb::stem::cmp(&t.name, &form.name) == std::cmp::Ordering::Equal)
        .next()
        .is_some();

    let fragment = owned_html!
    {
        @if !tags.is_empty()
        {
            h3
            {
                : "Search Results";
            }

            @for tag in tags
            {
                @if object.tags.iter().position(|obj_tag| obj_tag.tag_id == tag.tag_id).is_some()
                {
                    // Existing tag on this object

                    span(class="tag existing-tag")
                    {
                        : pages::templates::tags::render_existing(&tag);
                    }
                }
                else
                {
                    // New tag we can add to this object

                    a(href=format!(
                            "javascript:picvu.add_tag(decodeURIComponent('{}'), '{}', '{}', '{}');",
                            urlencoding::encode(&urlencoding::encode(&tag.name)),
                            tag.kind.to_string(),
                            tag.rating.num_stars().to_string(),
                            tag.censor.to_string()),
                        class="tag add-tag")
                    {
                        : OutlineIcon::PlusCircle.render(IconSize::Size16x16);
                        : pages::templates::tags::render_existing(&tag);
                    }
                }
            }
        }

        @if !form.name.is_empty() && !found_matching
        {
            h3
            {
                : format!("New Tag \"{}\"", form.name);
            }

            div(class="add-tag-sub-form")
            {
                h3
                {
                    : "Name";
                }

                a(href=format!(
                        "javascript:picvu.set_and_submit('hidden-add-tag-name', decodeURIComponent('{}'))",
                        urlencoding::encode(&urlencoding::encode(&form.name))),
                    class="tag add-tag")
                {
                    : OutlineIcon::PlusCircle.render(IconSize::Size16x16);
                    : form.name.clone();
                }

                : pages::templates::tag_kind::render("add-tag-kind", &picvudb::data::TagKind::Label);
                : pages::templates::rating::render("add-tag-rating", &picvudb::data::Rating::NotRated);
                : pages::templates::censor::render("add-tag-censor", &picvudb::data::Censor::FamilyFriendly);
            }
        }
        else
        {
            input(id="hidden-add-tag-kind", type="hidden", name="add_tag_kind", value="");
            input(id="hidden-add-tag-rating", type="hidden", name="add_tag_rating", value="");
            input(id="hidden-add-tag-censor", type="hidden", name="add_tag_censor", value="");
        }

    }.into_string().unwrap();

    Ok(view::html_fragment(fragment))
}

fn render_edit_object(state: &crate::State, object: picvudb::data::get::ObjectMetadata, all_objs_on_date: Vec<picvudb::data::get::ObjectMetadata>, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let filename = object.attachment.filename.clone();

    let title = view::Title
    {
        text: format!("Edit: {}", object.title.clone().map(|m| m.get_display_text()).unwrap_or(filename.clone())),
        html: Raw(format!("Edit: {}", object.title.clone().map(|m| m.get_html()).unwrap_or(owned_html!{ : filename.clone() }.into_string().unwrap()))),
    };

    let mut tag_suggestions = Vec::new();
    let mut seen_tags = HashSet::new();

    {
        let add_tags = |output: &mut Vec<(&'static str, Vec<picvudb::data::add::Tag>)>, 
            seen_tags: &mut HashSet<String>,
            name: &'static str,
            tags: Vec<picvudb::data::add::Tag>|
        {
            let mut final_tags = Vec::new();
            final_tags.reserve(tags.len());

            for tag in tags.into_iter()
            {
                let normalized = picvudb::stem::normalize(&tag.name);

                if seen_tags.contains(&normalized)
                {
                    // Already seen
                }
                else
                {
                    seen_tags.insert(normalized);
                    final_tags.push(tag);
                }
            }

            final_tags.sort_by(|a, b| picvudb::stem::cmp(&a.name, &b.name));

            output.push((name, final_tags));
        };

        let tags_on_same_day = all_objs_on_date.iter()
            .map(|obj| obj.tags.clone()) // Collect lists of tags from all objects on the same day
            .flatten()  // Flatten into a list of tags - with repetition
            .map(|tag| (tag.tag_id.clone(), tag)) // Prepare to insert into a map, by tag ID
            .collect::<std::collections::HashMap<_, _>>()   // Remove duplicates
            .values()   // Drop the tag IDs
            .filter(|tag| object.tags.iter().position(|otag| otag.tag_id == tag.tag_id).is_none())  // Only tags not already set for this object
            .filter(|tag| !tag.kind.is_system_kind())   // Filter out system tags
            .map(|tag| picvudb::data::add::Tag { name: tag.name.clone(), kind: tag.kind.clone(), rating: tag.rating.clone(), censor: tag.censor.clone(), }) // Map to add structure
            .collect();

        let recent =
        {
            let recent_tags = state.recent_tags.lock().unwrap();

            recent_tags.get_recent()
        };

        for tag in object.tags.iter()
        {
            seen_tags.insert(picvudb::stem::normalize(&tag.name));
        }

        add_tags(&mut tag_suggestions, &mut seen_tags, "On Same Day", tags_on_same_day);
        add_tags(&mut tag_suggestions, &mut seen_tags, "Recent", recent);
    }

    let index_of_this_obj = all_objs_on_date.iter()
        .position(|o| o.id == object.id)
        .unwrap_or(0);

    let contents = owned_html!
    {
        script(src="/assets/picvu.js");
        script(src="/assets/edit_object.js");

        form(id="form", method="POST", action=format!("/form/edit_object/{}", object.id.to_string()), enctype="application/x-www-form-urlencoded")
        {
            div(class="cmdbar cmdbar-top")
            {
                a(id="save", href="javascript:picvu.submit();", class="cmdbar-link")
                {
                    : OutlineIcon::Save.render(IconSize::Size16x16);
                    : " Save"
                }
                a(href=pages::object_details::ObjectDetailsPage::path_for(&object.id), class="cmdbar-link")
                {
                    : OutlineIcon::Cancel.render(IconSize::Size16x16);
                    : " Cancel"
                }
                a(href=pages::delete_object::DeleteObjectPage::path_for(&object.id), class="cmdbar-link")
                {
                    : OutlineIcon::Trash2.render(IconSize::Size16x16);
                    : " Delete"
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
                            format!("javascript:picvu.set_and_submit('hidden-next-object-id', '{}')", other_obj.id.to_string()),
                            other_obj.id == object.id);
                    }
                }
            }

            div(class="horiz-columns")
            {
                div
                {
                    h2
                    {
                        : "Preview";
                    }

                    a(href=pages::attachments::AttachmentsPage::path_attachment(&object.id, &object.attachment.hash))
                    {
                        : pages::attachments::AttachmentsPage::raw_html_for_thumbnail(&object, 512, true);
                    }
                }

                div
                {
                    h2
                    {
                        : "Edit Details";
                    }

                    input(id="hidden-object-id", type="hidden", value=object.id.to_string());

                    : pages::templates::rating::render("rating", &object.rating);
                    : pages::templates::censor::render("censor", &object.censor);

                    @if !object.tags.is_empty()
                    {
                        h3 { : "Remove Tags" }

                        @for tag in object.tags.iter()
                        {
                            a(href=format!("javascript:picvu.set_and_submit('hidden-remove-tag-id', '{}');", tag.tag_id.to_string()),
                                class="tag remove-tag")
                            {
                                : OutlineIcon::Cancel.render(IconSize::Size16x16);
                                : pages::templates::tags::render_existing(tag);
                            }
                        }
                    }

                    label(for="activity")
                    {
                        : "Date/Time";
                    }
                    input(id="edit-activity", type="text", name="activity", value=object.activity_time.to_rfc3339());

                    label(for="title")
                    {
                        : "Title";
                    }
                    input(id="edit-title", type="text", name="title", value=object.title.clone().map(|m| m.get_markdown()).unwrap_or_default());

                    label(for="location")
                    {
                        : "Location";
                    }
                    input(id="edit-location", type="text", name="location", value=object.location.clone().map(|l| l.to_string()).unwrap_or_default());

                    label(for="notes")
                    {
                        : "Notes";
                    }
                    textarea(id="edit-notes", name="notes", rows=10, cols=60)
                    {
                        : object.notes.clone().map(|m| m.get_markdown()).unwrap_or_default();
                    }
                }

                div
                {
                    h2
                    {
                        : "Add Tags";
                    }
                    input(id="hidden-remove-tag-id", type="hidden", name="remove_tag_id", value="");

                    label(for="search_tag_name")
                    {
                        : "Search";
                    }
                    input(id="edit-search-tag-name", type="search", name="search_tag_name", value="", autocomplete="off", placeholder="Search Tags");
                    input(id="hidden-add-tag-name", type="hidden", name="add_tag_name", value="");

                    div(id="add-tags-search-div")
                    {
                        input(id="hidden-add-tag-kind", type="hidden", name="add_tag_kind", value="");
                        input(id="hidden-add-tag-rating", type="hidden", name="add_tag_rating", value="");
                        input(id="hidden-add-tag-censor", type="hidden", name="add_tag_censor", value="");
                    }

                    @for (heading, suggestions) in tag_suggestions
                    {
                        @if !suggestions.is_empty()
                        {
                            h3 { : heading }

                            @for tag in suggestions.iter()
                            {
                                // Needs to be URI encoded as the web-browswer decodes the URI
                                // into the correct string.

                                a(href=format!(
                                        "javascript:picvu.add_tag(decodeURIComponent('{}'), '{}', '{}', '{}');",
                                        urlencoding::encode(&urlencoding::encode(&tag.name)),
                                        tag.kind.to_string(),
                                        tag.rating.num_stars().to_string(),
                                        tag.censor.to_string()),
                                    class="tag add-tag")
                                {
                                    : OutlineIcon::PlusCircle.render(IconSize::Size16x16);
                                    : pages::templates::tags::render_add(tag);
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
