use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::dev::HttpResponseBuilder;

use picvudb::msgs::GetObjectsResponse;
use picvudb::msgs::GetAttachmentDataResponse;
use picvudb::msgs::AddObjectResponse;

use crate::path;
use crate::bulk;

mod doc;
mod page;

pub use doc::redirect;

pub fn generate_response<T>(data: T) -> HttpResponse
    where T: Viewable
{
    data.generate()
}

pub fn generate_error_response<T>(builder: HttpResponseBuilder, data: T) -> HttpResponse
    where T: Debug
{
    doc::err(builder, data)
}

pub trait Viewable
{
    fn generate(self) -> HttpResponse;
}

impl Viewable for GetObjectsResponse
{
    fn generate(self) -> HttpResponse
    {
        doc::ok(page::objects(self))
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

impl<R, E> Viewable for Result<R, E>
    where R: Viewable,
        E: Debug
{
    fn generate(self) -> HttpResponse
    {
        match self
        {
            Ok(r) => r.generate(),
            Err(e) => doc::err(HttpResponse::InternalServerError(), e),
        }
    }
}
