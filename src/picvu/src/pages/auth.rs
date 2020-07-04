use serde::Deserialize;
use actix_web::{web, HttpResponse};

use crate::icons::Icon;
use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::pages;
use crate::State;

#[allow(dead_code)]
pub struct AuthPage
{
}

impl AuthPage
{
    pub fn path() -> String
    {
        "/auth/google/login".to_owned()
    }
}

impl PageResources for AuthPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .add_header_link("/auth/google/login", "Auth", Icon::Login, 500)
            .route_view("/auth/google/login", web::get().to(get_auth_login))
            .route_other("/auth/google/callback", web::get().to(get_auth_callback))
            .route_other("/auth/google/token", web::get().to(get_auth_token));
    }
}

async fn get_auth_login(state: web::Data<State>) -> Result<HttpResponse, view::ErrorResponder>
{
    let (client_id, client_secret) = pages::setup::get_client_id_and_secret(&*state).await?;

    if client_id.is_empty() || client_secret.is_empty()
    {
        return Ok(view::redirect(pages::setup::SetupPage::path()));
    }

    let redirect_url =
    {
        let setup = googlephotos::auth::GoogleAuthSetup
        {
            client_id,
            client_secret,
            redirect_url: format!("{}/auth/google/callback", state.host_base),
        };

        let mut google_auth_client = state.google_auth_client.lock().unwrap();

        google_auth_client.start_new(setup)
    };

    Ok(view::redirect(redirect_url))
}

#[derive(Deserialize)]
pub struct CallbackQuery
{
    pub error: Option<String>,
    pub code: Option<String>,
    pub state: Option<String>,
}

async fn get_auth_callback(state: web::Data<State>, form: web::Query<CallbackQuery>) -> Result<HttpResponse, view::ErrorResponder>
{
    {
        let mut google_auth_client = state.google_auth_client.lock().unwrap();

        google_auth_client.got_callback(form.code.clone(), form.state.clone(), form.error.clone());
    }

    Ok(view::redirect("/auth/google/token".to_owned()))
}

async fn get_auth_token(state: web::Data<State>) -> Result<HttpResponse, view::ErrorResponder>
{
    let operation =
    {
        let mut google_auth_client = state.google_auth_client.lock().unwrap();

        google_auth_client.exchange_token()
    };

    let token = web::block(||
    {
        operation.blocking_execute()
    }).await?;

    {
        let mut google_auth_client = state.google_auth_client.lock().unwrap();

        google_auth_client.save_token(token);
    };

    Ok(view::redirect("/".to_owned()))
}
