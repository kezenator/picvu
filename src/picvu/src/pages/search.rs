use serde::Deserialize;
use actix_web::{web, HttpResponse};

use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::pages;

#[allow(dead_code)]
pub struct SearchPage
{
}

impl SearchPage
{
    pub fn path() -> String
    {
        "/view/search".to_owned()
    }
}

impl PageResources for SearchPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/view/search", web::get().to(get_search));
    }
}

#[derive(Deserialize)]
struct SearchForm
{
    q: String,
}

async fn get_search(form: web::Query<SearchForm>) -> HttpResponse
{
    if let Ok(location) = form.q.parse()
    {
        // If we can decode the query as a location, then
        // redirect to the location page.

        let query = picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters: 100.0 };

        view::redirect(pages::object_listing::ObjectListingPage::path(query))
    }
    else
    {
        // Just treat it as a standard text search

        let query = picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ search: form.q.clone() };

        view::redirect(pages::object_listing::ObjectListingPage::path(query))
    }
}
