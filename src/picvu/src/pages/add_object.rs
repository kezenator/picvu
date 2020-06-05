use futures::{StreamExt, TryStreamExt};
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;


use crate::analyse;
use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::pages;

#[allow(dead_code)]
pub struct AddObjectPage
{
}

impl AddObjectPage
{
    pub fn post_path() -> String
    {
        "/form/add_object".to_owned()
    }
}

impl PageResources for AddObjectPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_other("/form/add_object", web::post().to(post_add_object));

    }
}

async fn post_add_object(state: web::Data<State>, mut payload: Multipart) -> Result<HttpResponse, view::ErrorResponder>
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

    Ok(view::redirect(pages::object_details::ObjectDetailsPage::path_for(&response.object_id)))
}
