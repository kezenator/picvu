use std::collections::HashMap;
use picvudb::data::add::Tag;
use picvudb::data::get::TagMetadata;

pub struct RecentTagCache
{
    lru_order: Vec<String>,
    tags: HashMap<String, Tag>,
}

impl RecentTagCache
{
    pub fn new() -> Self
    {
        RecentTagCache
        {
            lru_order: Vec::new(),
            tags: HashMap::new(),
        }
    }

    pub fn get_recent(&self) -> Vec<Tag>
    {
        let mut result = self.tags.values().cloned().collect::<Vec<_>>();

        result.sort_by(|a, b|  picvudb::stem::cmp(&a.name, &b.name));

        result
    }

    pub fn add_existing(&mut self, existing: &TagMetadata)
    {
        let tag = Tag
        {
            name: existing.name.clone(),
            kind: existing.kind.clone(),
            rating: existing.rating.clone(),
            censor: existing.censor.clone(),
        };

        self.add_new(&tag);
    }

    pub fn add_new(&mut self, tag: &Tag)
    {
        let normalized = picvudb::stem::normalize(&tag.name);

        if normalized == picvudb::stem::normalize(&picvudb::data::TagKind::system_name_trash())
            || normalized == picvudb::stem::normalize(&picvudb::data::TagKind::system_name_unsorted())
        {
            // Don't add the special system names
            return;
        }

        // Remove the specified tag

        self.remove(&normalized);

        // Ensure there's not too many entries

        if self.lru_order.len() >= 10
        {
            let oldest = self.lru_order[self.lru_order.len() - 1].clone();

            self.remove(&oldest);
        }

        // Add the new entry in

        self.lru_order.push(normalized.clone());
        self.tags.insert(normalized, tag.clone());
    }

    fn remove(&mut self, norm: &str)
    {
        let mut new_lru = Vec::new();
        let mut new_tags = HashMap::new();

        std::mem::swap(&mut self.lru_order, &mut new_lru);
        std::mem::swap(&mut self.tags, &mut new_tags);

        let new_lru = new_lru.into_iter()
            .filter(|n| n != norm)
            .collect();

        let new_tags = new_tags.into_iter()
            .filter(|(key, _val)| key != norm)
            .collect();

        self.lru_order = new_lru;
        self.tags = new_tags;
    }
}