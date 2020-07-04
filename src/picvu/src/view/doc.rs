use std::fmt::Debug;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::header::{
        CacheControl, CacheDirective,
        Charset,
        ContentDisposition, ContentType,
        DispositionType, DispositionParam,
        ExtendedValue,
        EntityTag, ETag};

use horrorshow::{owned_html, Raw, Template};

use crate::icons::{Icon, IconSize};
use crate::pages::HeaderLinkCollection;

pub fn err<T>(builder: HttpResponseBuilder, err: T) -> HttpResponse
    where T: Debug
{
    let body = owned_html!{
        h1: "Error";
        pre
        {
            : format!("{:?}", err);
        }
    }.into_string().unwrap();

    html_response(builder, "Error", &body)
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

pub fn html_response(builder: HttpResponseBuilder, title: &str, body: &str) -> HttpResponse
{
    let mut builder = builder;

    let body = owned_html!
    {
        : Raw("<!DOCTYPE html>");

        html(lang="en")
        {
            head
            {
                meta(charset="utf-8");
                link(rel="stylesheet", href="/assets/style.css");

                title : title
            }
            body
            {
                : Raw(body)
            }
        }
    }.into_string().unwrap();

    builder
        .set(ContentType::html())
        .set(CacheControl(vec![
            CacheDirective::NoStore,
        ]))
        .body(body.to_string())
}

pub fn html_page(req: &HttpRequest, header_links: &HeaderLinkCollection, title: &str, icon: Icon, content: &str) -> HttpResponse
{
    let body = owned_html!{
        : header(title, icon, req, header_links);
        : Raw(content)
    }.into_string().unwrap();

    html_response(HttpResponse::Ok(), title, &body)
}

pub fn header(title: &str, icon: Icon, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Raw<String>
{
    let html = owned_html!{

        div(class="header")
        {
            h1
            {
                : icon.render(IconSize::Size32x32);
                : title;
            }

            div(class="header-links")
            {
                @for header in header_links.by_order()
                {
                    a(href=(&header.path),
                      class=(if header.path == req.path() { Some("header-link-selected") } else { None }))
                    {
                        : header.icon.render(IconSize::Size16x16);
                        : header.label;
                    }
                }

                form(method="GET", action=crate::pages::search::SearchPage::path(), enctype="application/x-www-form-urlencoded")
                {
                    : Icon::Search.render(IconSize::Size16x16);
                    input(type="search", name="q");
                    input(type="submit", value="Search");
                }
            }
        }

    }.into_string().unwrap();

    Raw(html)
}
