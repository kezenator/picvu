use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::header::{
        CacheControl, CacheDirective,
        Charset,
        ContentDisposition, ContentType,
        DispositionType, DispositionParam,
        ExtendedValue,
        EntityTag, ETag};

use horrorshow::{html, Raw};

use crate::view::page::Page;

pub fn ok(page: Page) -> HttpResponse
{
    html_response(HttpResponse::Ok(), page)
}

pub fn err<T>(builder: HttpResponseBuilder, err: T) -> HttpResponse
    where T: Debug
{
    let page = Page
    {
        title: "Error".to_owned(),
        contents: html!{ pre : format!("{:?}", err) }.to_string(),
    };

    html_response(builder, page)
}

pub fn redirect(path: String) -> HttpResponse
{
    HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, path)
        .finish()
}

pub fn binary(bytes: Vec<u8>, filename: String, mime: mime::Mime, etag: String) -> HttpResponse
{
    HttpResponse::Ok()
    .set(ContentType(mime))
    .set(CacheControl(vec![
        CacheDirective::Public,
        CacheDirective::MaxAge(24 * 3600 /* 1 day */),
        CacheDirective::Extension("immutable".to_owned(), None),
    ]))
    .set(ETag(EntityTag::strong(etag)))
    .set(ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
            charset: Charset::Ext("UTF-8".to_owned()),
            language_tag: None,
            value: filename.bytes().collect::<Vec<u8>>(),
        })],
    })
    .body(bytes)
}

fn html_response(builder: HttpResponseBuilder, page: Page) -> HttpResponse
{
    let mut builder = builder;

    let body = html!
    {
        html
        {
            head
            {
                title : page.title.as_str()
            }
            body : Raw(page.contents.as_str())
        }
    };

    builder
        .set(ContentType::html())
        .set(CacheControl(vec![
            CacheDirective::NoStore,
        ]))
        .body(body.to_string())
}
