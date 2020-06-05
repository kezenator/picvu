extern crate actix_rt;
use std::sync::{Arc, Mutex};
use actix::SyncArbiter;
use actix_web::{web, App, HttpServer, HttpResponse};
use googlephotos::auth::GoogleAuthClient;

mod analyse;
mod assets;
mod bulk;
mod db;
mod format;
mod pages;
mod view;

use pages::PageResources;

pub struct State {
    host_base: String,
    bulk_queue: Arc<Mutex<bulk::BulkQueue>>,
    db: db::DbAddr,
    db_uri: String,
    google_auth_client: Arc<Mutex<GoogleAuthClient>>,
    header_links: pages::HeaderLinkCollection,
}

async fn get_index() -> HttpResponse
{
    view::redirect(pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::ByActivityDesc))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>
{

    let db_uri = "E:\\test.db";

    // TODO - better file handling
    //let _remove_err = std::fs::remove_file(db_uri);

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("picvu-db");

        let addr = SyncArbiter::start(1, move || {
            db::DbExecutor::new(picvudb::Store::new(db_uri).expect("Could not open DB"))
        });

        tx.send(addr).unwrap();

        sys.run().expect("Cannot run picvu-db system");
    });

    let addr = rx.recv().unwrap();
    let bulk_queue = Arc::new(Mutex::new(bulk::BulkQueue::new()));
    let google_auth_client = Arc::new(Mutex::new(GoogleAuthClient::new()));

    HttpServer::new(move ||
    {
        let mut page_builder = pages::PageResourcesBuilder::new();

        pages::object_details::ObjectDetailsPage::page_resources(&mut page_builder);
        pages::object_listing::ObjectListingPage::page_resources(&mut page_builder);
        pages::attachments::AttachmentsPage::page_resources(&mut page_builder);
        pages::setup::SetupPage::page_resources(&mut page_builder);
        pages::auth::AuthPage::page_resources(&mut page_builder);
        pages::sync::SyncPage::page_resources(&mut page_builder);
        pages::add_object::AddObjectPage::page_resources(&mut page_builder);
        pages::bulk::BulkPage::page_resources(&mut page_builder);

        let state = State
        {
            host_base: "http://localhost:8080".to_owned(),
            bulk_queue: bulk_queue.clone(),
            db: db::DbAddr::new(addr.clone()),
            db_uri: db_uri.to_owned(),
            google_auth_client: google_auth_client.clone(),
            header_links: page_builder.header_links,
        };

        let mut app = App::new()
            .data(state)
            .route("/", web::get().to(get_index))
            .route("/assets/{_:.*}", web::get().to(assets::handle_embedded_file));
        
        for resource in page_builder.view_resources
        {
            app = app.service(resource);
        }

        for resource in page_builder.other_resources
        {
            app = app.service(resource);
        }

        app
    })
    .bind("localhost:8080")?
    .run()
    .await
}
