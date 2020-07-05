use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Template};

use crate::analyse;
use crate::bulk;
use crate::icons::OutlineIcon;
use crate::pages;
use crate::pages::{PageResources, PageResourcesBuilder};
use crate::State;
use crate::view;

#[allow(dead_code)]
pub struct BulkPage
{
}

impl BulkPage
{
    pub fn progress_path() -> String
    {
        "/view/bulk-progress".to_owned()
    }
}

impl PageResources for BulkPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/view/bulk_import", "Import", OutlineIcon::Import, 600)
            .route_view("/view/bulk_import", web::get().to(get_bulk_import))
            .route_other("/view/bulk-progress", web::get().to(get_bulk_progress))
            .route_other("/form/bulk_import", web::post().to(post_bulk_import))
            .route_other("/form/bulk_acknowledge", web::post().to(post_bulk_acknowledge));
    }
}

#[derive(Deserialize)]
pub struct BulkImportForm
{
    pub folder: String,
    pub assume_timezone: String,
    pub force_timezone: String,
    pub assume_notes: String,
    pub assume_location: String,
}

fn parse_str_to_opt<T: std::str::FromStr>(s: &str) -> Result<Option<T>, HttpResponse>
{
    if s.is_empty()
    {
        Ok(None)
    }
    else if let Ok(val) = s.parse()
    {
        Ok(Some(val))
    }
    else
    {
        Err(view::err(HttpResponse::BadRequest(), "Invalid parameter"))
    }
}

async fn post_bulk_import(state: web::Data<State>, form: web::Form<BulkImportForm>) -> Result<HttpResponse, HttpResponse>
{
    let api_key = pages::setup::get_api_key(&*state).await.unwrap_or_default();

    let access_token =
    {
        let auth = state.google_auth_client.lock().unwrap();

        match auth.access_token()
        {
            Some(access_token) =>
            {
                access_token
            },
            None =>
            {
                return Ok(view::redirect(pages::auth::AuthPage::path()));
            },
        }
    };

    let import_options = analyse::import::ImportOptions
    {
        assume_timezone: parse_str_to_opt(&form.assume_timezone)?,
        force_timezone: parse_str_to_opt(&form.force_timezone)?,
        assume_notes: parse_str_to_opt(&form.assume_notes)?,
        assume_location: parse_str_to_opt(&form.assume_location)?,
    };

    {
        let mut bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.enqueue(bulk::import::FolderImport::new(form.folder.clone(), state.db_uri.clone(), api_key, access_token, import_options));
    }

    Ok(view::redirect(BulkPage::progress_path()))
}

async fn post_bulk_acknowledge(state: web::Data<State>) -> HttpResponse
{
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.remove_completed();
    }

    view::redirect("/".to_owned())
}

fn get_bulk_import(state: web::Data<State>, req: HttpRequest) -> HttpResponse
{
    let contents = owned_html!
    {
        h1: "Add New Object";
        form(method="POST", action=pages::add_object::AddObjectPage::post_path(), enctype="multipart/form-data")
        {
            input(type="file", name="file", accept="image/*,video/*");
            input(type="submit");
        }
        h1: "Bulk Import";
        form(method="POST", action="/form/bulk_import", enctype="application/x-www-form-urlencoded")
        {
            h2: "Import Folder";
            em: "The path to the local folder that contains the media files.";
            p
            {
                input(type="text", name="folder")
            }

            h2: "Assume Timezone";
            p { em: "The timezone that is assumed if none is present. This is useful for photos without a GPS time-stamp, as both a GPS UTC timestamp and an associated local timestamp are required to determine the timezone."; }
            p { em: "Set this to the (incorrect) timezone the camera was set to."; }
            p
            {
                input(type="text", name="assume_timezone")
            }

            h2: "Force Timezone";
            em: "Forces all times to be displayed in the specified timezone. This is useful if the photos were taken on a camera that was in the wrong timezone - e.g. home local time while traviling away.";
            p { em: "Set this to the actual local timezone you were in when the photos were taken."; }
            p
            {
                input(type="text", name="force_timezone");
            }

            h2: "Assume Notes";
            em: "Assumes some notes to apply to any files that don't already contain notes.";
            p
            {
                input(type="text", name="assume_notes");
            }

            h2: "Assume Location";
            em: "Assumes a GPS location to be tagged to any files that don't already contain one.";
            p
            {
                input(type="text", name="assume_location");
            }

            p
            {
                input(type="submit");
            }
        }
    }.into_string().unwrap();

    view::html_page(
        &req,
        &state.header_links,
        "Import",
        OutlineIcon::Import,
        &contents)
}

fn get_bulk_progress(state: web::Data<State>, req: HttpRequest) -> HttpResponse
{
    let bulk_queue = state.bulk_queue.lock().unwrap();

    match bulk_queue.get_current_progress()
    {
        None =>
        {
            view::redirect("/".to_owned())
        },
        Some(progress) =>
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
                        form(method="POST", action="/form/bulk_acknowledge", enctype="application/x-www-form-urlencoded")
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

            view::html_page(&req, &state.header_links, "Bulk Operations", OutlineIcon::FilePlus, &contents)
        },
    }
}
