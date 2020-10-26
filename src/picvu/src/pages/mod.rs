use actix_web::{web, Resource};
use std::collections::BTreeMap;

use crate::icons::Icon;

pub mod add_object;
pub mod attachments;
pub mod auth;
pub mod bulk;
pub mod delete_object;
pub mod edit_object;
pub mod object_details;
pub mod object_listing;
pub mod search;
pub mod setup;
pub mod sync;
pub mod tags;

pub struct HeaderLink
{
    pub path: String,
    pub label: String,
    pub icon: Icon,
}

pub struct HeaderLinkCollection
{
    by_order: BTreeMap<(isize, String), (String, Icon)>,
}

impl HeaderLinkCollection
{
    pub fn new() -> Self
    {
        HeaderLinkCollection
        {
            by_order: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, path: String, label: String, icon: Icon, order: isize)
    {
        self.by_order.insert((order, path), (label, icon));
    }

    pub fn by_order(&self) -> Vec<HeaderLink>
    {
        self.by_order
            .iter()
            .map(|(k, v)| { HeaderLink { path: k.1.clone(), label: v.0.clone(), icon: v.1.clone(), }})
            .collect()
    }
}

pub trait PageResources
{
    fn page_resources(builder: &mut PageResourcesBuilder);
}

pub struct PageResourcesBuilder
{
    pub header_links: HeaderLinkCollection,
    pub view_resources: Vec<Resource>,
    pub other_resources: Vec<Resource>,
}

impl PageResourcesBuilder
{
    pub fn new() -> Self
    {
        PageResourcesBuilder
        {
            header_links: HeaderLinkCollection::new(),
            view_resources: Vec::new(),
            other_resources: Vec::new(),
        }
    }

    pub fn add_header_link<I: Into<Icon>>(&mut self, path: &str, label: &str, icon: I, order: isize) -> &mut Self
    {
        let path = path.to_owned();
        let label = label.to_owned();

        self.header_links.insert(path, label, icon.into(), order);
        self
    }

    pub fn route_view(&mut self, path: &str, route: actix_web::Route) -> &mut Self
    {
        self.view_resources.push(web::resource(path).route(route));
        self
    }

    pub fn route_other(&mut self, path: &str, route: actix_web::Route) -> &mut Self
    {
        self.other_resources.push(web::resource(path).route(route));
        self
    }
}
