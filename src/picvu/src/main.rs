extern crate actix_rt;
use std::sync::{Arc, Mutex};
use actix::SyncArbiter;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse};
use futures::{StreamExt, TryStreamExt};

mod analyse;
mod assets;
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

async fn object_query(state: web::Data<State>, pagination: &forms::Pagination, query: picvudb::data::get::GetObjectsQuery, view_type: view::derived::ViewObjectsListType) -> Result<HttpResponse, view::ErrorResponder>
{
    // TODO - this should be put into a middle-ware
    // that wraps all user-interface pages
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        if let Some(progress) = bulk_queue.get_current_progress()
        {
            return Ok(view::generate_response(progress));
        }
    }

    let pagination = picvudb::data::get::PaginationRequest
    {
        offset: pagination.offset.unwrap_or(0),
        page_size: pagination.page_size.unwrap_or(25),
    };

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination,
    };

    let response = state.db.send(msg).await??;

    // If we asked for one object, and it's a photo,
    // see if we can also get the attachment data
    // so that we can print out the EXIF data

    if let picvudb::data::get::GetObjectsQuery::ByObjectId(_) = &response.query
    {
        if let Some(object) = response.objects.first().clone()
        {
            let get_attachment_data_msg = picvudb::msgs::GetAttachmentDataRequest{
                object_id: object.id.clone(),
                specific_hash: None,
            };

            let attachment_response = state.db.send(get_attachment_data_msg).await;
            
            if let Ok(Ok(picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes})) = attachment_response
            {
                return Ok(view::generate_response(view::derived::ViewSingleObject {
                    object: object.clone(),
                    image_analysis: analyse::img::ImgAnalysis::decode(&bytes, &metadata.filename),
                }));
            }
        }
    }

    // Otherwise, just generate the general listing response

    Ok(view::generate_response(
        view::derived::ViewObjectsList { response, view_type }))
}

async fn objects_by_activity_desc(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &pagination_query, picvudb::data::get::GetObjectsQuery::ByActivityDesc, view::derived::ViewObjectsListType::ThumbnailsGrid).await
}

async fn objects_by_modified_desc(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &pagination_query, picvudb::data::get::GetObjectsQuery::ByModifiedDesc, view::derived::ViewObjectsListType::ThumbnailsGrid).await
}

async fn objects_by_size_desc(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &pagination_query, picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc, view::derived::ViewObjectsListType::ThumbnailsGrid).await
}

async fn objects_details_list(state: web::Data<State>, pagination_query: web::Query<forms::Pagination>) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &pagination_query, picvudb::data::get::GetObjectsQuery::ByActivityDesc, view::derived::ViewObjectsListType::DetailsTable).await
}

async fn object_details(state: web::Data<State>, object_id: web::Path<String>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());
    let pagination = forms::Pagination{ offset: None, page_size: None };

    object_query(state, &pagination, picvudb::data::get::GetObjectsQuery::ByObjectId(object_id), view::derived::ViewObjectsListType::ThumbnailsGrid).await
}

async fn form_add_object(state: web::Data<State>, mut payload: Multipart, _req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let mut file: Option<(String, Vec<u8>)> = None;

    loop
    {
        let section = payload.try_next().await?;

        match section
        {
            None => break,
            Some(mut field) =>
            {
                if let Some(content_type) = field.content_disposition()
                {
                    if let Some(filename) = content_type.get_filename()
                    {
                        let mut bytes: Vec<u8> = Vec::new();

                        while let Some(chunk) = field.next().await
                        {
                            let chunk = chunk?;

                            bytes.extend_from_slice(&chunk);
                        }

                        file = Some((filename.to_owned(), bytes));
                    }
                }
            },
        }
    }

    let multipart_err = actix_multipart::MultipartError::Payload(actix_http::error::PayloadError::Incomplete(Some(
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Request is missing a file")
    )));

    let (file_name, bytes) = file.ok_or(multipart_err)?;

    // We ignore warnings here

    let mut warnings = Vec::new();

    let add_msg = analyse::import::create_add_object_for_import(
        bytes,
        &file_name,
        None,
        None,
        None,
        &mut warnings)?;

    let response = state.db.send(add_msg).await??;
    Ok(view::generate_response(response))
}

async fn form_bulk_import(state: web::Data<State>, form: web::Form<forms::BulkImport>, _req: HttpRequest) ->HttpResponse
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

async fn attachment(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<forms::Attachment>, _req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;
    Ok(view::generate_response(response))
}

async fn thumbnail(state: web::Data<State>, path: web::Path<String>, form: web::Query<forms::Thumbnail>, _req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(path.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;

    let response = match response
    {
        picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound =>
        {
            picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound
        },
        picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
        {
            picvudb::msgs::GetAttachmentDataResponse::HashNotFound
        },
        picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes} =>
        {
            web::block(move || -> Result<picvudb::msgs::GetAttachmentDataResponse, image::ImageError>
            {
                let orientation =
                    analyse::img::ImgAnalysis::decode(&bytes, &metadata.filename)
                    .ok()
                    .flatten()
                    .map(|(analysis, _warnings)|{ analysis.orientation })
                    .flatten();

                let image = image::load_from_memory(&bytes)?;
                let image = image.thumbnail(form.size, form.size);

                let image = match orientation
                {
                    None
                        | Some(analyse::img::Orientation::Straight) =>
                    {
                        image
                    },
                    Some(analyse::img::Orientation::UpsideDown) =>
                    {
                        image.rotate180()
                    }
                    Some(analyse::img::Orientation::RotatedLeft) =>
                    {
                        image.rotate90()
                    }
                    Some(analyse::img::Orientation::RotatedRight) =>
                    {
                        image.rotate270()
                    }
                };

                let mut bytes = Vec::new();
                image.write_to(&mut bytes, image::ImageOutputFormat::Jpeg(100))?;

                Ok(picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes})
            }).await?
        },
    };

    Ok(view::generate_response(response))
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
            .route("/", web::get().to(objects_by_activity_desc))
            .route("/assets/{_:.*}", web::get().to(assets::handle_embedded_file))
            .route("/view/object/{obj_id}", web::get().to(object_details))
            .route("/view/objects/by_modified_desc", web::get().to(objects_by_modified_desc))
            .route("/view/objects/by_activity_desc", web::get().to(objects_by_activity_desc))
            .route("/view/objects/by_size_desc", web::get().to(objects_by_size_desc))
            .route("/view/objects/details_list", web::get().to(objects_details_list))
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
