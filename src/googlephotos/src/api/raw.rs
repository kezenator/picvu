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

pub fn media_items_search_album(access_token: &auth::AccessToken, album_id: String, next_page_token: Option<String>) -> Result<MediaItemsListResponse, GoogleApiError>
{
    let url = Url::parse("https://photoslibrary.googleapis.com/v1/mediaItems:search").unwrap();

    let mut list = List::new();
    list.append(&format!("Authorization: Bearer {}", urlencoding::encode(&access_token.secret()))).unwrap();
    list.append("Content-Type: application/json").unwrap();

    let search = SearchRequest
    {
        album_id: Some(album_id),
        page_size: 100,
        page_token: next_page_token,
    };

    let req_body = serde_json::to_vec(&search)?;

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;
    handle.post(true)?;
    handle.post_fields_copy(&req_body)?;
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

pub fn albums_list(access_token: &auth::AccessToken, next_page_token: Option<String>) -> Result<AlbumsListResponse, GoogleApiError>
{
    let mut url = Url::parse("https://photoslibrary.googleapis.com/v1/albums").unwrap();

    url.query_pairs_mut().append_pair("pageSize", "50");

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

    let body = serde_json::from_slice::<AlbumsListResponse>(&data)?;

    Ok(body)
}

pub fn albums_create(access_token: &auth::AccessToken, album_name: String) -> Result<String, GoogleApiError>
{
    let url = Url::parse("https://photoslibrary.googleapis.com/v1/albums").unwrap();

    let mut list = List::new();
    list.append(&format!("Authorization: Bearer {}", urlencoding::encode(&access_token.secret()))).unwrap();
    list.append("Content-Type: application/json").unwrap();

    let request = CreateAlbumRequest
    {
        album: CreateAlbumInfo
        {
            title: album_name,
        },
    };

    let req_body = serde_json::to_vec(&request)?;

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;
    handle.post(true)?;
    handle.post_fields_copy(&req_body)?;
    handle.http_headers(list)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }

    let body = serde_json::from_slice::<Album>(&data)?;

    Ok(body.id)
}

pub fn albums_bulk_add(access_token: &auth::AccessToken, album_id: String, media_item_ids: Vec<String>) -> Result<(), GoogleApiError>
{
    let url = Url::parse(&format!("https://photoslibrary.googleapis.com/v1/albums/{}:batchAddMediaItems", album_id)).unwrap();

    let mut list = List::new();
    list.append(&format!("Authorization: Bearer {}", urlencoding::encode(&access_token.secret()))).unwrap();
    list.append("Content-Type: application/json").unwrap();

    let request = AlbumModifyRequest
    {
        media_item_ids,
    };

    let req_body = serde_json::to_vec(&request)?;

    println!("=========================================");
    println!("{}", String::from_utf8_lossy(&req_body));

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;
    handle.post(true)?;
    handle.post_fields_copy(&req_body)?;
    handle.http_headers(list)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }

    if !data.is_empty()
    {
        Err(GoogleApiError::new_unexpected_response(String::from_utf8_lossy(&data).to_string()))
    }
    else
    {
        Ok(())
    }
}