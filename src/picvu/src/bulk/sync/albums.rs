use std::collections::{HashMap, HashSet};

use googlephotos::auth::AccessToken;

use crate::bulk::progress::ProgressSender;
use crate::bulk::sync::SyncError;

#[derive(Debug)]
pub struct AlbumDatabase
{
    albums: HashMap<String, AlbumInfo>,
    name_to_album_id: HashMap<String, String>,
}

impl AlbumDatabase
{
    pub fn load_all(access_token: &AccessToken, sender: &ProgressSender) -> Result<AlbumDatabase, SyncError>
    {
        // First, load all of the albums

        let mut raw_albums = Vec::new();
        let mut total_album_entries: u64 = 0;

        {
            let mut next_page_token = None;

            loop
            {
                sender.set(0.0, vec![format!("Loaded {} albums", raw_albums.len())]);

                let response = googlephotos::api::raw::albums_list(access_token, next_page_token)?;

                next_page_token = response.next_page_token;

                for raw_album in response.albums
                {
                    total_album_entries += raw_album.media_items_count.clone().unwrap_or_default().parse::<u64>().ok().unwrap_or_default();
                    raw_albums.push(raw_album);
                }

                if next_page_token.is_none()
                {
                    break;
                }
            }
        }

        // Now load each album's media items,
        // and create a hash map

        let mut albums = HashMap::new();
        let mut name_to_album_id = HashMap::new();

        let num_albums = raw_albums.len();
        let mut albums_loaded: usize = 0;
        let mut entries_loaded: u64 = 0;

        for raw_album in raw_albums
        {
            albums_loaded += 1;

            let mut media_ids = Vec::new();

            {
                let mut next_page_token = None;

                loop
                {
                    sender.set(
                        (entries_loaded as f64) / (total_album_entries as f64) * 100.0,
                        vec![
                            format!("Loading {} of {} albums", albums_loaded, num_albums),
                            format!("Loaded {} of {} entries (total)", entries_loaded, total_album_entries)
                        ]);

                    let response = googlephotos::api::raw::media_items_search_album(access_token, raw_album.id.clone(), next_page_token)?;
    
                    next_page_token = response.next_page_token;

                    if let Some(media_items) = response.media_items
                    {
                        entries_loaded += media_items.len() as u64;

                        for media_item in media_items
                        {
                            media_ids.push(media_item.id);
                        }
                    }

                    if next_page_token.is_none()
                    {
                        break;
                    }
                }
            }

            let album_info = AlbumInfo
            {
                title: raw_album.title.clone(),
                contents: media_ids,
            };

            albums.insert(raw_album.id.clone(), album_info);
            name_to_album_id.insert(raw_album.title, raw_album.id);
        }

        Ok(AlbumDatabase{ albums, name_to_album_id })
    }

    pub fn create_albums(&mut self, album_names: HashSet<String>, access_token: &AccessToken, sender: &ProgressSender) -> Result<(), SyncError>
    {
        // First, for out which albums we actually need to create
        // by removing ones that we already know about

        let mut album_names = album_names;

        for (name, _id) in self.name_to_album_id.iter()
        {
            album_names.remove(name);
        }

        // Now, actually create each item

        let num_to_create = album_names.len();
        let mut num_created: usize = 0;

        for name in album_names
        {
            num_created += 1;

            sender.set(
                (num_created as f64) / (num_to_create as f64) * 100.0,
                vec!
                [
                    format!("Creating {} of {} new albums", num_created, num_to_create),
                    format!("Creating {:?}", name),
                ]);

            let id = googlephotos::api::raw::albums_create(access_token, name.clone())?;

            assert!(!self.albums.contains_key(&id));

            let album_info = AlbumInfo
            {
                title: name.clone(),
                contents: Vec::new(),
            };

            self.albums.insert(id.clone(), album_info);
            self.name_to_album_id.insert(name, id);
        }

        Ok(())
    }

    pub fn apply_changes(&mut self, memberships: HashMap<String, Vec<String>>, access_token: &AccessToken, sender: &ProgressSender) -> Result<(), SyncError>
    {
        let mut memberships = memberships;

        let mut changes = Vec::new();

        for (id, album_info) in self.albums.iter()
        {
            let cur_contents = &album_info.contents;
            let wanted_contents = memberships.remove(&album_info.title).unwrap_or(Vec::new());

            calc_changes(&mut changes, id, &album_info.title, cur_contents, &wanted_contents);
        }

        println!("Changes:\n{:#?}", changes);

        let num_changes = changes.len();
        let mut changes_performed: usize = 0;

        for change in changes
        {
            changes_performed += 1;

            sender.set(
                (changes_performed as f64) / (num_changes as f64) * 100.0,
                vec![
                    format!("Performing {} or {} changes", changes_performed, num_changes),
                    format!("Updating album {:?}", change.album_title)
                ]);

            if change.is_add
            {
                googlephotos::api::raw::albums_bulk_add(access_token, change.album_id, change.media_item_ids)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct AlbumInfo
{
    title: String,
    contents: Vec<String>,
}

#[derive(Debug)]
struct Change
{
    is_add: bool,
    album_id: String,
    album_title: String,
    media_item_ids: Vec<String>,
}

fn calc_changes(changes: &mut Vec<Change>, album_id: &String, album_title: &String, cur_contents: &Vec<String>, wanted_contents: &Vec<String>)
{
    if cur_contents != wanted_contents
    {
        if !cur_contents.is_empty()
        {
            changes.push(Change
                {
                    is_add: false,
                    album_id: album_id.clone(),
                    album_title: album_title.clone(),
                    media_item_ids: cur_contents.clone()
                });
        }

        if !wanted_contents.is_empty()
        {
            changes.push(Change
                {
                    is_add: true,
                    album_id: album_id.clone(),
                    album_title: album_title.clone(),
                    media_item_ids: wanted_contents.clone()
                });
        }
    }
}