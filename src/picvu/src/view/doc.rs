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

use crate::icons::{Icon, IconSize, OutlineIcon};
use crate::pages::HeaderLinkCollection;

#[derive(Clone)]
pub struct Title
{
    pub text: String,
    pub html: Raw<String>,
}

impl<T> From<T> for Title
    where T: Into<String>
{
    fn from(s: T) -> Self
    {
        let text: String = s.into();
        let text2 = text.clone();

        let html = Raw(owned_html!{ : text2 }.into_string().unwrap());

        Title { text, html }
    }
}

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

pub fn html_response<T: Into<Title>>(builder: HttpResponseBuilder, title: T, body: &str) -> HttpResponse
{
    let title: Title = title.into();

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

                title : title.html
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

pub fn html_page<T: Into<Title>, I: Into<Icon>>(req: &HttpRequest, header_links: &HeaderLinkCollection, title: T, icon: I, content: &str) -> HttpResponse
{
    let title: Title = title.into();
    let title_text = title.text.clone();

    let body = owned_html!{
        : header(&title, icon.into(), req, header_links);
        : Raw(content)
    }.into_string().unwrap();

    html_response(HttpResponse::Ok(), &title_text, &body)
}

pub fn header<I: Into<Icon>>(title: &Title, icon: I, req: &HttpRequest, header_links: &HeaderLinkCollection) -> Raw<String>
{
    let title = title.clone();
    let icon: Icon = icon.into();

    let html = owned_html!{

        div(class="header")
        {
            h1
            {
                : icon.render(IconSize::Size32x32);
                : &title.html;
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
                    : OutlineIcon::Search.render(IconSize::Size16x16);
                    input(type="search", name="q");
                    input(type="submit", value="Search");
                }
            }
        }

    }.into_string().unwrap();

    Raw(html)
}
