use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::header::ContentType;

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
        .body(body.to_string())
}
