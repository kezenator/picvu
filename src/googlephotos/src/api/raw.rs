use curl::easy::{Easy, List};
use url::Url;

use crate::auth;
use super::msgs::*;
use super::GoogleApiError;

pub fn media_items_list(access_token: &auth::AccessToken, next_page_token: Option<String>) -> Result<MediaItemsListResponse, GoogleApiError>
{
    let mut url = Url::parse("https://photoslibrary.googleapis.com/v1/mediaItems").unwrap();

    url.query_pairs_mut().append_pair("pageSize", "100");

    if let Some(next_page_token) = next_page_token
    {
        url.query_pairs_mut().append_pair("pageToken", &next_page_token);
    }

    let mut list = List::new();
    list.append(&format!("Authorization: Bearer {}", urlencoding::encode(&access_token.secret()))).unwrap();

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;
    handle.http_headers(list)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }

    let body = serde_json::from_slice::<MediaItemsListResponse>(&data)?;

    Ok(body)
}
