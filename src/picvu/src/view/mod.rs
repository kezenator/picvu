use std::fmt::Debug;
use actix_web::HttpResponse;

use picvudb::msgs::GetAllObjectsResponse;
use picvudb::msgs::AddObjectResponse;

use crate::path;

mod doc;
mod page;

pub fn generate_response<T>(data: T) -> HttpResponse
    where T: Viewable
{
    data.generate()
}

pub trait Viewable
{
    fn generate(&self) -> HttpResponse;
}

impl Viewable for GetAllObjectsResponse
{
    fn generate(&self) -> HttpResponse
    {
        doc::ok(page::all_objects(self))
    }
}

impl Viewable for AddObjectResponse
{
    fn generate(&self) -> HttpResponse
    {
        doc::redirect(path::index())
    }
}

impl<R, E> Viewable for Result<R, E>
    where R: Viewable,
        E: Debug
{
    fn generate(&self) -> HttpResponse
    {
        match &self
        {
            Ok(r) => r.generate(),
            Err(e) => doc::err(HttpResponse::InternalServerError(), e),
        }
    }
}
