use actix_web::{web, HttpResponse};

use crate::bulk;
use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::pages;
use crate::State;

#[allow(dead_code)]
pub struct SyncPage
{
}

impl PageResources for SyncPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/sync/start", "Sync", 600)
            .route_view("/sync/start", web::get().to(get_sync_start));
    }
}

async fn get_sync_start(state: web::Data<State>) -> Result<HttpResponse, view::ErrorResponder>
{
    let access_token =
    {
        let auth = state.google_auth_client.lock().unwrap();

        auth.access_token()
    };

    match access_token
    {
        Some(access_token) =>
        {
            let mut bulk_queue = state.bulk_queue.lock().unwrap();

            bulk_queue.enqueue(bulk::sync::GooglePhotosSync::new(access_token, state.db_uri.clone()));

            Ok(view::redirect("/".to_owned()))
        },
        None =>
        {
            Ok(view::redirect(pages::auth::AuthPage::path()))
        },
    }
}
