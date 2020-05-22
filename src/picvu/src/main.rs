extern crate actix_rt;
use std::sync::{Arc, Mutex};
use actix::SyncArbiter;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse};
use futures::{StreamExt, TryStreamExt};

mod analyse;
mod bulk;
mod db;
mod format;
mod forms;
mod path;
mod view;

struct State {
    bulk_queue: Arc<Mutex<bulk::BulkQueue>>,
    db: db::DbAddr,
    db_uri: String,
}

async fn object_query(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>, query: picvudb::data::get::GetObjectsQuery) -> HttpResponse
{
    // TODO - this should be put into a middle-ware
    // that wraps all user-interface pages
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        if let Some(progress) = bulk_queue.get_current_progress()
        {
            return view::generate_response(progress);
        }
    }

    let pagination = picvudb::data::get::PaginationRequest
    {
        offset: pagination_query.offset.unwrap_or(0),
        page_size: pagination_query.page_size.unwrap_or(25),
    };

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination,
    };

    let response = state.db.send(msg).await;
    view::generate_response(response)
}

async fn objects_by_modified_desc(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> HttpResponse
{
    object_query(state, pagination_query, picvudb::data::get::GetObjectsQuery::ByModifiedDesc).await
}

async fn objects_by_size_desc(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> HttpResponse
{
    object_query(state, pagination_query, picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc).await
}

async fn form_add_object(state: web::Data<State>, mut payload: Multipart, _req: HttpRequest) -> HttpResponse
{
    let mut file: Option<(String, Vec<u8>)> = None;

    loop
    {
        match payload.try_next().await
        {
            Err(err) => return view::generate_error_response(HttpResponse::BadRequest(), err),
            Ok(None) => break,
            Ok(Some(mut field)) =>
            {
                if let Some(content_type) = field.content_disposition()
                {
                    if let Some(filename) = content_type.get_filename()
                    {
                        let mut bytes: Vec<u8> = Vec::new();

                        while let Some(chunk) = field.next().await
                        {
                            match chunk
                            {
                                Err(err) =>return view::generate_error_response(HttpResponse::BadRequest(), err),
                                Ok(chunk) =>
                                {
                                    bytes.extend_from_slice(&chunk);
                                }
                            }
                        }

                        file = Some((filename.to_owned(), bytes));
                    }
                }
            },
        }
    }

    if file.is_none()
    {
        return view::generate_error_response(HttpResponse::BadRequest(), "no file provided");
    }
    let file = file.unwrap();
    let now = picvudb::data::Date::now();

    {
        let details = analyse::img::ImgAnalysis::decode(&file.1, &file.0);
        println!("Image Analysis Details:");
        println!("{:#?}", details);
    }

    let data = picvudb::data::add::ObjectData
    {
        title: Some(file.0.clone()),
        additional: picvudb::data::add::AdditionalData::Photo
        {
            attachment: picvudb::data::add::Attachment
            {
                filename: file.0.clone(),
                created: now.clone(),
                modified: now.clone(),
                mime: mime::IMAGE_JPEG,
                bytes: file.1,
            },
        }
    };

    let msg = picvudb::msgs::AddObjectRequest{ data };
    let response = state.db.send(msg).await;
    view::generate_response(response)
}

async fn form_bulk_import(state: web::Data<State>, form: web::Form<forms::BulkImport>, _req: HttpRequest) -> HttpResponse
{
    {
        let mut bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.enqueue(bulk::import::FolderImport::new(form.folder.clone(), state.db_uri.clone()));
    }

    view::redirect(path::index())
}

async fn form_bulk_acknowledge(state: web::Data<State>, _req: HttpRequest) -> HttpResponse
{
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.remove_completed();
    }

    view::redirect(path::index())
}

async fn attachment(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<forms::Attachment>, _req: HttpRequest) -> HttpResponse
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await;
    view::generate_response(response)
}

async fn thumbnail(state: web::Data<State>, path: web::Path<String>, form: web::Query<forms::Thumbnail>, _req: HttpRequest) -> HttpResponse
{
    let object_id = picvudb::data::ObjectId::new(path.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await;

    if response.is_err()
    {
        return view::generate_response(response);
    }
    let response = response.unwrap();

    if response.is_err()
    {
        return view::generate_response(response);
    }
    let response = response.unwrap();

    let response = match response
    {
        picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound =>
        {
            Ok(picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound)
        },
        picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
        {
            Ok(picvudb::msgs::GetAttachmentDataResponse::HashNotFound)
        },
        picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes} =>
        {
            web::block(move || -> Result<picvudb::msgs::GetAttachmentDataResponse, image::ImageError>
            {
                let orientation =
                    analyse::img::ImgAnalysis::decode(&bytes, &metadata.filename)
                    .ok()
                    .flatten()
                    .map(|analysis|{ analysis.orientation })
                    .unwrap_or(analyse::img::Orientation::Undefined);

                let image = image::load_from_memory(&bytes)?;
                let image = image.thumbnail(form.size, form.size);

                let image = match orientation
                {
                    analyse::img::Orientation::Undefined
                    | analyse::img::Orientation::Straight =>
                    {
                        image
                    },
                    analyse::img::Orientation::UpsideDown =>
                    {
                        image.rotate180()
                    }
                    analyse::img::Orientation::RotatedLeft =>
                    {
                        image.rotate90()
                    }
                    analyse::img::Orientation::RotatedRight =>
                    {
                        image.rotate270()
                    }
                };

                let mut bytes = Vec::new();
                image.write_to(&mut bytes, image::ImageOutputFormat::Jpeg(100))?;

                Ok(picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes})
            }).await
        },
    };

    view::generate_response(response)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>
{

    let db_uri = "test.db";

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

    HttpServer::new(move ||
    {
        let state = State
        {
            bulk_queue: bulk_queue.clone(),
            db: db::DbAddr::new(addr.clone()),
            db_uri: db_uri.to_owned(),
        };

        App::new()
            .data(state)
            .route("/", web::get().to(objects_by_modified_desc))
            .route("/view/objects/by_modified_desc", web::get().to(objects_by_modified_desc))
            .route("/view/objects/by_size_desc", web::get().to(objects_by_size_desc))
            .route("/form/add_object", web::post().to(form_add_object))
            .route("/form/bulk_import", web::post().to(form_bulk_import))
            .route("/form/bulk_acknowledge", web::post().to(form_bulk_acknowledge))
            .route("/attachments/{object_id}", web::get().to(attachment))
            .route("/thumbnails/{object_id}", web::get().to(thumbnail))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
