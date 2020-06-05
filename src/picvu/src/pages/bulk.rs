use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Template};

use crate::pages::{PageResources, PageResourcesBuilder};
use crate::bulk;
use crate::view;
use crate::State;

#[allow(dead_code)]
pub struct BulkPage
{
}

impl BulkPage
{
    pub fn path() -> String
    {
        "/view/bulk-progress".to_owned()
    }

    pub fn bulk_import_path() -> String
    {
        "/form/bulk_import".to_owned()
    }
}

impl PageResources for BulkPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_other("/view/bulk-progress", web::get().to(get_bulk_progress))
            .route_other("/form/bulk_import", web::post().to(post_bulk_import))
            .route_other("/form/bulk_acknowledge", web::post().to(post_bulk_acknowledge));
    }
}

#[derive(Deserialize)]
pub struct BulkImportForm
{
    pub folder: String,
}

async fn post_bulk_import(state: web::Data<State>, form: web::Form<BulkImportForm>) ->HttpResponse
{
    {
        let mut bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.enqueue(bulk::import::FolderImport::new(form.folder.clone(), state.db_uri.clone()));
    }

    view::redirect(BulkPage::path())
}

async fn post_bulk_acknowledge(state: web::Data<State>) -> HttpResponse
{
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.remove_completed();
    }

    view::redirect("/".to_owned())
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

            view::html_page(&req, &state.header_links, "Bulk Operations", &contents)
        },
    }
}
