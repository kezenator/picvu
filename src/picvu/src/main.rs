extern crate actix_rt;
use actix::SyncArbiter;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse, Responder};

mod db;

struct State {
    db: db::DbAddr,
}

async fn greet(state: web::Data<State>, _req: HttpRequest) -> impl Responder {

    let msg = picvudb::msgs::GetPropertiesRequest{};
    let response = state.db.send(msg).await.unwrap();
    match response
    {
        Ok(properties) => HttpResponse::Ok().json(properties),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("picvu-db");

        // Start 3 parallel db executors
        let addr = SyncArbiter::start(3, || {
            db::DbExecutor::new(picvudb::Store::new(":memory:").expect("Could not open DB"))
        });

        tx.send(addr).unwrap();

        sys.run().expect("Cannot run picvu-db system");
    });

    let addr = rx.recv().unwrap();

    HttpServer::new(move || {
        App::new()
            .data(State { db: db::DbAddr::new(addr.clone()) })
            .route("/", web::get().to(greet))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
