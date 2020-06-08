use std::sync::{Arc, Mutex};
use actix::SyncArbiter;
use actix_web::{web, App, HttpServer, HttpResponse};
use structopt::StructOpt;

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

#[derive(Debug, StructOpt)]
#[structopt(name = "picvu", about = "A locally-hosted website for managing photos, including Google Photos integration")]
struct CmdArgs
{
    /// The database file name to use
    #[structopt(short, long, default_value="picvu.db")]
    file: String,
    /// The hostname to serve the web-site from
    #[structopt(short, long, default_value="localhost:8080")]
    host: String,
}

async fn get_index() -> HttpResponse
{
    view::redirect(pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::ByActivityDesc))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>
{
    let args = CmdArgs::from_args();

    let db_uri1 = args.file.clone();
    let db_uri2 = args.file;
    let host_base = format!("http://{}", args.host);

    // TODO - better file handling
    //let _remove_err = std::fs::remove_file(db_uri);

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("picvu-db");

        let addr = SyncArbiter::start(1, move || {
            db::DbExecutor::new(picvudb::Store::new(&db_uri1).expect("Can't open database"))
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
            host_base: host_base.clone(),
            bulk_queue: bulk_queue.clone(),
            db: db::DbAddr::new(addr.clone()),
            db_uri: db_uri2.clone(),
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
