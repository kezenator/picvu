use std::fmt::Debug;
use actix_web::HttpResponse;
use picvudb::msgs::GetPropertiesResponse;

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

impl Viewable for GetPropertiesResponse
{
    fn generate(&self) -> HttpResponse
    {
        doc::ok(page::properties(self))
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
