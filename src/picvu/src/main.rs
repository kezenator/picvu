extern crate actix_rt;
use std::sync::{Arc, Mutex};
use actix::SyncArbiter;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use googlephotos::auth::GoogleAuthClient;

mod analyse;
mod assets;
mod bulk;
mod db;
mod format;
mod forms;
mod path;
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

async fn object_query(state: web::Data<State>, options: &forms::ListViewOptions, query: picvudb::data::get::GetObjectsQuery, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    // TODO - this should be put into a middle-ware
    // that wraps all user-interface pages
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        if let Some(progress) = bulk_queue.get_current_progress()
        {
            return Ok(view::generate_response(progress, &req, &state.header_links));
        }
    }

    let pagination = picvudb::data::get::PaginationRequest
    {
        offset: options.offset.unwrap_or(0),
        page_size: options.page_size.unwrap_or(25),
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
                    mvimg_split: analyse::img::parse_mvimg_split(&bytes, &metadata.filename),
                },
                &req,
                &state.header_links));
            }
        }
    }

    // Otherwise, just generate the general listing response

    Ok(view::generate_response(view::derived::ViewObjectsList
    {
        response: response,
        list_type: options.list_type.unwrap_or(view::derived::ViewObjectsListType::ThumbnailsGrid),
    },
    &req,
    &state.header_links))
}

async fn objects_by_activity_desc(state: web::Data<State>, options: web::Query<forms::ListViewOptions>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByActivityDesc, req).await
}

async fn objects_by_modified_desc(state: web::Data<State>, options: web::Query<forms::ListViewOptions>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByModifiedDesc, req).await
}

async fn objects_by_size_desc(state: web::Data<State>, options: web::Query<forms::ListViewOptions>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc, req).await
}

async fn object_details(state: web::Data<State>, object_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());
    let options = forms::ListViewOptions{ list_type: None, offset: None, page_size: None };

    object_query(state, &options, picvudb::data::get::GetObjectsQuery::ByObjectId(object_id), req).await
}

async fn form_add_object(state: web::Data<State>, mut payload: Multipart, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
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
    Ok(view::generate_response(response, &req, &state.header_links))
}

async fn form_bulk_import(state: web::Data<State>, form: web::Form<forms::BulkImport>) ->HttpResponse
{
    {
        let mut bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.enqueue(bulk::import::FolderImport::new(form.folder.clone(), state.db_uri.clone()));
    }

    view::redirect(path::index())
}

async fn form_bulk_acknowledge(state: web::Data<State>) -> HttpResponse
{
    {
        let bulk_queue = state.bulk_queue.lock().unwrap();

        bulk_queue.remove_completed();
    }

    view::redirect(path::index())
}

async fn attachment(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<forms::Attachment>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;
    Ok(view::generate_response(response, &req, &state.header_links))
}

async fn thumbnail(state: web::Data<State>, path: web::Path<String>, form: web::Query<forms::Thumbnail>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
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

    Ok(view::generate_response(response, &req, &state.header_links))
}

async fn mvimg(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<forms::MvImg>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let mut response = state.db.send(msg).await??;

    if let picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes} = &response
    {
        let bytes = bytes[form.mp4_offset..].to_vec();
        let mut metadata = metadata.clone();
        metadata.mime = "video/mp4".parse::<mime::Mime>().unwrap();

        response = picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes};
    }

    Ok(view::generate_response(response, &req, &state.header_links))
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

        page_builder.add_header_link(
            &path::objects(picvudb::data::get::GetObjectsQuery::ByActivityDesc),
            "Calendar",
            0);
        page_builder.add_header_link(
            &path::objects(picvudb::data::get::GetObjectsQuery::ByModifiedDesc),
            "Recently Modified",
            1);
        page_builder.add_header_link(
            &path::objects(picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc),
            "Largest Attachments",
            2);

        pages::setup::SetupPage::page_resources(&mut page_builder);
        pages::auth::AuthPage::page_resources(&mut page_builder);
        pages::sync::SyncPage::page_resources(&mut page_builder);

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
            .route("/", web::get().to(objects_by_activity_desc))
            .route("/assets/{_:.*}", web::get().to(assets::handle_embedded_file))
            .route("/view/object/{obj_id}", web::get().to(object_details))
            .route("/view/objects/by_modified_desc", web::get().to(objects_by_modified_desc))
            .route("/view/objects/by_activity_desc", web::get().to(objects_by_activity_desc))
            .route("/view/objects/by_size_desc", web::get().to(objects_by_size_desc))
            .route("/form/add_object", web::post().to(form_add_object))
            .route("/form/bulk_import", web::post().to(form_bulk_import))
            .route("/form/bulk_acknowledge", web::post().to(form_bulk_acknowledge))
            .route("/attachments/{object_id}", web::get().to(attachment))
            .route("/thumbnails/{object_id}", web::get().to(thumbnail))
            .route("/mvimgs/{object_id}", web::get().to(mvimg));
        
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
