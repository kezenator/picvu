use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::dev::HttpResponseBuilder;
use actix_web::ResponseError;

use picvudb::msgs::GetAttachmentDataResponse;
use picvudb::msgs::AddObjectResponse;

use crate::path;
use crate::bulk;

pub mod derived;
mod doc;
mod page;

pub use doc::redirect;

#[derive(Debug)]
pub enum ErrorResponder
{
    ActixMailboxError(actix::MailboxError),
    PicvudbError(picvudb::Error),
    MultipartError(actix_multipart::MultipartError),
    StdIoError(std::io::Error),
    BlockingOperationCanceled,
    ImageError(image::error::ImageError),
}

impl std::fmt::Display for ErrorResponder
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{:?}", self)
    }
}

impl From<actix::MailboxError> for ErrorResponder
{
    fn from(error: actix::MailboxError) -> Self
    {
        ErrorResponder::ActixMailboxError(error)
    }
}

impl From<picvudb::Error> for ErrorResponder
{
    fn from(error: picvudb::Error) -> Self
    {
        ErrorResponder::PicvudbError(error)
    }
}

impl From<actix_multipart::MultipartError> for ErrorResponder
{
    fn from(error: actix_multipart::MultipartError) -> Self
    {
        ErrorResponder::MultipartError(error)
    }
}

impl From<std::io::Error> for ErrorResponder
{
    fn from(error: std::io::Error) -> Self
    {
        ErrorResponder::StdIoError(error)
    }
}

impl<T> From<actix_web::error::BlockingError<T>> for ErrorResponder
    where T: Into<ErrorResponder> + Debug
{
    fn from(error: actix_web::error::BlockingError<T>) -> Self
    {
        match error
        {
            actix_web::error::BlockingError::Canceled => ErrorResponder::BlockingOperationCanceled,
            actix_web::error::BlockingError::Error(e) => e.into(),
        }
    }
}

impl From<image::error::ImageError> for ErrorResponder
{
    fn from(error: image::error::ImageError) -> Self
    {
        ErrorResponder::ImageError(error)
    }
}

impl ResponseError for ErrorResponder
{
    fn error_response(&self) -> HttpResponse
    {
        let builder = HttpResponseBuilder::new(self.status_code());
        let contents = format!("{:#?}", self);

        doc::err(builder, contents)
    }

    fn status_code(&self) -> StatusCode
    {
        match self
        {
            Self::ActixMailboxError(_)
                | Self::PicvudbError(_)
                | Self::StdIoError(_) 
                | Self::BlockingOperationCanceled
                | Self::ImageError(_) =>
            {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Self::MultipartError(_) =>
            {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

pub fn generate_response<T>(data: T) -> HttpResponse
    where T: Viewable
{
    data.generate()
}

pub trait Viewable
{
    fn generate(self) -> HttpResponse;
}

impl Viewable for derived::ViewObjectsList
{
    fn generate(self) -> HttpResponse
    {
        if self.response.pagination_request.offset != self.response.pagination_response.offset
            || self.response.pagination_request.page_size != self.response.pagination_response.page_size
        {
            // Redirect to the index with the correct pagesize
            doc::redirect(path::objects_with_pagination(
                self.response.query,
                self.response.pagination_response.offset,
                self.response.pagination_request.page_size))
        }
        else
        {
            match self.view_type
            {
                derived::ViewObjectsListType::ThumbnailsGrid =>
                    doc::ok(page::objects_thumbnails(self.response)),

                derived::ViewObjectsListType::DetailsTable =>
                    doc::ok(page::objects_details(self.response))
            }
        }
    }
}

impl Viewable for derived::ViewSingleObject
{
    fn generate(self) -> HttpResponse
    {
        doc::ok(page::object_details(self.object, self.image_analysis))
    }
}

impl Viewable for GetAttachmentDataResponse
{
    fn generate(self) -> HttpResponse
    {
        match self
        {
            GetAttachmentDataResponse::ObjectNotFound =>
            {
                doc::err(HttpResponse::NotFound(), "Object not found")
            },
            GetAttachmentDataResponse::HashNotFound =>
            {
                doc::err(HttpResponse::NotFound(), "Object not found")
            },
            GetAttachmentDataResponse::Found{metadata, bytes} =>
            {
                doc::binary(bytes, metadata.filename, metadata.mime, metadata.hash)
            },
        }
    }
}

impl Viewable for AddObjectResponse
{
    fn generate(self) -> HttpResponse
    {
        doc::redirect(path::index())
    }
}

impl Viewable for bulk::progress::ProgressState
{
    fn generate(self) -> HttpResponse
    {
        doc::ok(page::bulk_progress(self))
    }
}
