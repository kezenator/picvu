use serde::Deserialize;
use actix_web::{web, HttpResponse};

use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::analyse;

#[allow(dead_code)]
pub struct AttachmentsPage
{
}

impl AttachmentsPage
{
    pub fn path_attachment(obj_id: &picvudb::data::ObjectId, hash: &String) -> String
    {
        format!("/attachments/{}/raw?hash={}", obj_id.to_string(), hash)
    }

    pub fn path_image_thumbnail(obj_id: &picvudb::data::ObjectId, hash: &String, size: u32) -> String
    {
        format!("/attachments/{}/img_thumb?hash={}&size={}", obj_id.to_string(), hash, size)
    }

    pub fn path_mvimg(obj_id: &picvudb::data::ObjectId, hash: &String) -> String
    {
        format!("/attachments/{}/mvimg?hash={}", obj_id.to_string(), hash)
    }
}

impl PageResources for AttachmentsPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_other("/attachments/{object_id}/raw", web::get().to(get_attachment))
            .route_other("/attachments/{object_id}/img_thumb", web::get().to(get_img_thumbnail))
            .route_other("/attachments/{object_id}/mvimg", web::get().to(get_mvimg));
    }
}

#[derive(Deserialize)]
pub struct FormAttachment
{
    pub hash: String,
}

#[derive(Deserialize)]
pub struct FormThumbnail
{
    pub hash: String,
    pub size: u32,
}

#[derive(Deserialize)]
pub struct FormMvImg
{
    pub hash: String,
}

async fn get_attachment(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<FormAttachment>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;

    match response
    {
        picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object not found"))
        }
        picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object's current attachment has a different hash"))
        }
        picvudb::msgs::GetAttachmentDataResponse::Found{bytes, metadata} =>
        {
            Ok(view::binary(bytes, metadata.filename, metadata.mime, metadata.hash))
        }
    }
}

async fn get_img_thumbnail(state: web::Data<State>, path: web::Path<String>, form: web::Query<FormThumbnail>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(path.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;

    match response
    {
        picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object not found"))
        }
        picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object's current attachment has a different hash"))
        }
        picvudb::msgs::GetAttachmentDataResponse::Found{bytes, metadata} =>
        {
            let (bytes, metadata) = web::block(move || -> Result<(Vec<u8>, picvudb::data::get::AttachmentMetadata), image::ImageError>
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

                Ok((bytes, metadata))
            }).await?;

            Ok(view::binary(bytes, metadata.filename, mime::IMAGE_JPEG, metadata.hash))
        },
    }
}

async fn get_mvimg(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<FormMvImg>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::new(object_id.to_string());

    let msg = picvudb::msgs::GetAttachmentDataRequest{ object_id, specific_hash: Some(form.hash.clone()) };
    let response = state.db.send(msg).await??;

    match response
    {
        picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object not found"))
        }
        picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Object's current attachment has a different hash"))
        }
        picvudb::msgs::GetAttachmentDataResponse::Found{bytes, metadata} =>
        {
            let mvimg_info = analyse::img::parse_mvimg_split(&bytes, &metadata.filename);

            match mvimg_info
            {
                analyse::img::MvImgSplit::Neither =>
                {
                    Ok(view::err(HttpResponse::NotFound(), "Object is not a motion JPEG image"))
                },
                analyse::img::MvImgSplit::JpegOnly =>
                {
                    Ok(view::err(HttpResponse::NotFound(), "Object is JPEG only - there is no movie component"))
                },
                analyse::img::MvImgSplit::Mp4Only =>
                {
                    Ok(view::binary(bytes, metadata.filename, "video/mp4".parse().unwrap(), metadata.hash))
                },
                analyse::img::MvImgSplit::Both{mp4_offset} =>
                {
                    let mp4_bytes = bytes[mp4_offset..].to_vec();

                    Ok(view::binary(mp4_bytes, metadata.filename, "video/mp4".parse().unwrap(), metadata.hash))
                },
            }
        }
    }
}
