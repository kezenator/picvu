use serde::Deserialize;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::pages::{PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
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
    pub list_type: Option<pages::object_listing::ViewObjectsListType>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

async fn get_search(state: web::Data<State>, form: web::Query<SearchForm>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    if let Ok(location) = form.q.parse()
    {
        // If we can decode the query as a location, then
        // redirect to the location page.

        let query = picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters: 100.0 };
        return Ok(view::redirect(pages::object_listing::ObjectListingPage::path(query)));
    }

    // TODO - search by string
    let query = picvudb::data::get::GetObjectsQuery::ByActivityDesc;

    let options = pages::object_listing::ListViewOptionsForm
    {
        list_type: form.list_type,
        offset: form.offset,
        page_size: form.page_size,
    };

    pages::object_listing::object_query(state, &options, query, req).await
}
