use std::collections::HashMap;
use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Template};

use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;

#[allow(dead_code)]
pub struct SetupPage
{
}

impl SetupPage
{
    pub fn path() -> String
    {
        "/view/setup".to_owned()
    }
}

impl PageResources for SetupPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/view/setup", "Setup", 400)
            .route_view("/view/setup", web::get().to(get_setup_form))
            .route_other("/forms/setup", web::post().to(post_setup_form));
    }
}

async fn get_api_key_client_id_and_secret(state: &State) -> Result<(String, String, String), view::ErrorResponder>
{
    let mut properties = state.db.send(picvudb::msgs::GetPropertiesRequest{}).await??;

    let api_key = properties.properties.remove(PROP_NAME_API_KEY).unwrap_or_default();
    let client_id = properties.properties.remove(PROP_NAME_CLIENT_ID).unwrap_or_default();
    let client_secret = properties.properties.remove(PROP_NAME_CLIENT_SECRET).unwrap_or_default();

    Ok((api_key, client_id, client_secret))
}

pub async fn get_api_key(state: &State) -> Result<String, view::ErrorResponder>
{
    let (api_key, _client_id, _client_secret) = get_api_key_client_id_and_secret(&*state).await?;

    Ok(api_key)
}

pub async fn get_client_id_and_secret(state: &State) -> Result<(String, String), view::ErrorResponder>
{
    let (_api_key, client_id, client_secret) = get_api_key_client_id_and_secret(&*state).await?;

    Ok((client_id, client_secret))
}

async fn get_setup_form(state: web::Data<State>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let (api_key, client_id, client_secret) = get_api_key_client_id_and_secret(&*state).await?;

    let contents = owned_html!
    {
        form(method="POST", action="/forms/setup", enctype="application/x-www-form-urlencoded")
        {
            p { : "Google Maps (Geocoding/TimeZone) API Key" }
            input(type="text", name="api_key", value=api_key);

            p { : "Google Photos Client ID" }
            input(type="text", name="client_id", value=client_id);

            p { : "Google Photos Client Secret" }
            input(type="password", name="client_secret");

            @if !client_secret.is_empty()
            {
                p
                {
                    i: format!("{} {}",
                        "A client secret has already been configured. It is not shown here.",
                        "You must re-enter the client secret each time you submit this form.");
                }
            }

            input(type="submit");
        }
    }.into_string().unwrap();

    Ok(view::html_page(&req, &state.header_links, "Setup", &contents))
}

const PROP_NAME_API_KEY: &'static str = "api_key.auth.google.com";
const PROP_NAME_CLIENT_ID: &'static str = "client_id.auth.google.com";
const PROP_NAME_CLIENT_SECRET: &'static str = "client_secret.auth.google.com";

#[derive(Deserialize)]
pub struct SetupForm
{
    pub api_key: String,
    pub client_id: String,
    pub client_secret: String,
}

async fn post_setup_form(state: web::Data<State>, form: web::Form<SetupForm>) -> Result<HttpResponse, view::ErrorResponder>
{
    let mut properties = HashMap::new();

    properties.insert(PROP_NAME_API_KEY.to_owned(), form.api_key.clone());
    properties.insert(PROP_NAME_CLIENT_ID.to_owned(), form.client_id.clone());
    properties.insert(PROP_NAME_CLIENT_SECRET.to_owned(), form.client_secret.clone());

    let _ = state.db.send(picvudb::msgs::SetPropertiesRequest{ properties }).await??;

    Ok(view::redirect("/view/setup".to_owned()))
}
