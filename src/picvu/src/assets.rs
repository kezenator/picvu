use actix_web::body::Body;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header::{CacheControl, CacheDirective};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn handle_embedded_file(req: HttpRequest) -> HttpResponse
{
    // Trim the leading "/assets/" of the path
    let path = &req.path()["/assets/".len()..];

    match Assets::get(path)
    {
        Some(content) =>
        {
            let body: Body = match content.data
            {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok()
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .set(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(3600),
                ]))
        
                .body(body)
        },
        None =>
        {
            HttpResponse::NotFound()
                .body("404 Not Found")
        },
    }
  }
  