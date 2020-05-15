extern crate actix_rt;
use actix::SyncArbiter;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse};

mod db;
mod forms;
mod path;
mod view;

struct State {
    db: db::DbAddr,
}

async fn index(state: web::Data<State>, _req: HttpRequest) -> HttpResponse {

    let msg = picvudb::msgs::GetAllObjectsRequest{};
    let response = state.db.send(msg).await;
    view::generate_response(response)
}

async fn form_add_object(state: web::Data<State>, form: web::Form<forms::AddObject>, _req: HttpRequest) -> HttpResponse {
    let msg = picvudb::msgs::AddObjectRequest{ label: form.label.clone() };
    let response = state.db.send(msg).await;
    view::generate_response(response)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("picvu-db");

        // Start 3 parallel db executors
        let addr = SyncArbiter::start(1, || {
            db::DbExecutor::new(picvudb::Store::new(":memory:").expect("Could not open DB"))
        });

        tx.send(addr).unwrap();

        sys.run().expect("Cannot run picvu-db system");
    });

    let addr = rx.recv().unwrap();

    HttpServer::new(move || {
        App::new()
            .data(State { db: db::DbAddr::new(addr.clone()) })
            .route("/", web::get().to(index))
            .route("/form/add_object", web::post().to(form_add_object))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
