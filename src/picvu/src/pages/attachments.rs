use serde::Deserialize;
use actix_web::{web, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use crate::analyse;
use crate::pages::{PageResources, PageResourcesBuilder};
use crate::State;
use crate::view;

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

    pub fn path_video_thumbnail(obj_id: &picvudb::data::ObjectId, hash: &String, size: u32) -> String
    {
        format!("/attachments/{}/video_thumb?hash={}&size={}", obj_id.to_string(), hash, size)
    }

    pub fn raw_html_for_thumbnail(object: &picvudb::data::get::ObjectMetadata, size: u32, play_video: bool) -> Raw<String>
    {
        calc_raw_html_for_thumbnail(object, size, play_video)
    }
}

impl PageResources for AttachmentsPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_other("/attachments/{object_id}/raw", web::get().to(get_attachment))
            .route_other("/attachments/{object_id}/img_thumb", web::get().to(get_img_thumbnail))
            .route_other("/attachments/{object_id}/video_thumb", web::get().to(get_video_thumbnail))
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
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

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
    let object_id = picvudb::data::ObjectId::try_new(path.to_string())?;

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
                    analyse::img::ImgAnalysis::decode(&bytes, &metadata.filename, None)
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

async fn get_video_thumbnail(state: web::Data<State>, path: web::Path<String>, form: web::Query<FormThumbnail>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(path.to_string())?;

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
            let filename = metadata.filename;

            let info = web::block(move || -> Result<analyse::video::VideoAnalysisResults, std::io::Error>
            {
                let assume_timezone = None;
                let mut warnings = Vec::new();

                analyse::video::analyse_video(&bytes, &filename, form.size, &assume_timezone, None, &mut warnings)
            }).await?;

            match info.thumbnail
            {
                None =>
                {
                    Ok(view::err(HttpResponse::NotFound(), "Can't generate video thumbnail"))
                },
                Some(thumbnail) =>
                {
                    Ok(view::binary(
                        thumbnail.bytes,
                        thumbnail.filename,
                        thumbnail.mime,
                        metadata.hash))
                },
            }
        },
    }
}

async fn get_mvimg(state: web::Data<State>, object_id: web::Path<String>, form: web::Query<FormMvImg>) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

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

fn calc_raw_html_for_thumbnail(object: &picvudb::data::get::ObjectMetadata, size: u32, play_video: bool) -> Raw<String>
{
    let dimensions = object.attachment.dimensions.clone().map(|d| d.resize_to_max_dimension(size));

    Raw(owned_html!
    {
        @if object.attachment.mime == mime::IMAGE_GIF
        {
            @if let Some(dimensions) = dimensions
            {
                img(src=AttachmentsPage::path_attachment(&object.id, &object.attachment.hash),
                    width=dimensions.width.to_string(),
                    height=dimensions.height.to_string())
            }
            else
            {
                // No dimensions - just try as a re-sized thumbnail of the correct size
                img(src=AttachmentsPage::path_image_thumbnail(&object.id, &object.attachment.hash, size))
            }
        }
        else if object.attachment.mime.type_() == mime::IMAGE
        {
            @if let Some(dimensions) = dimensions
            {
                img(src=AttachmentsPage::path_image_thumbnail(&object.id, &object.attachment.hash, size),
                    width=dimensions.width.to_string(),
                    height=dimensions.height.to_string())
            }
            else
            {
                // No dimensions - just try as a re-sized thumbnail of the correct size
                img(src=AttachmentsPage::path_image_thumbnail(&object.id, &object.attachment.hash, size))
            }
        }
        else if object.attachment.mime.type_() == mime::VIDEO
        {
            @if let Some(dimensions) = dimensions
            {
                @if play_video
                {
                    video(width=dimensions.width.to_string(),
                        height=dimensions.height.to_string(),
                        autoplay="true",
                        muted="true",
                        controls="true",
                        loop="true")
                    {
                        source(
                            src=AttachmentsPage::path_attachment(&object.id, &object.attachment.hash),
                            type=object.attachment.mime.to_string())
                    }
                }
                else
                {
                    img(src=AttachmentsPage::path_video_thumbnail(&object.id, &object.attachment.hash, size),
                        width=dimensions.width.to_string(),
                        height=dimensions.height.to_string())
                }
            }
            else
            {
                div: "Video";
            }

        }
        else
        {
            div: "Other";
        }
    }.into_string().unwrap())
}
