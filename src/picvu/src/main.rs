extern crate actix_rt;
use actix::SyncArbiter;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse};
use futures::{StreamExt, TryStreamExt};

mod analyse;
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

async fn form_add_object(state: web::Data<State>, mut payload: Multipart, _req: HttpRequest) -> HttpResponse {

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
async fn main() -> std::io::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("picvu-db");

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
            .route("/attachments/{object_id}", web::get().to(attachment))
            .route("/thumbnails/{object_id}", web::get().to(thumbnail))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
