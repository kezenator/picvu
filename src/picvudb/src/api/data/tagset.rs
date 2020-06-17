use crate::ParseError;
use crate::data::TagId;

pub struct TagSet(Vec<i64>);

impl TagSet
{
    pub(crate) fn to_db_field(&self) -> Option<String>
    {
        if self.0.is_empty()
        {
            None
        }
        else
        {
            Some(self.0
                .iter()
                .map(|id| TagId::from_db_field(*id).to_string())
                .collect::<Vec<_>>()
                .join(" "))
        }
    }

    pub(crate) fn to_db_vec(&self) -> &Vec<i64>
    {
        &self.0
    }

    pub(crate) fn from_db_set(val: &std::collections::BTreeSet<i64>) -> Self
    {
        TagSet(val.iter().map(|t| *t).collect())
    }

    pub(crate) fn from_db_field(val: Option<String>) -> Result<Self, ParseError>
    {
        match val
        {
            None =>
            {
                Ok(TagSet(Vec::new()))
            },
            Some(val) =>
            {
                let mut vec_orig_order: Vec<TagId> = Vec::new();
                for s in val.split(' ')
                {
                    vec_orig_order.push(s.parse().map_err(|_| ParseError::new(format!("Invalid TagSet string: {:?}", val)))?);
                }
                
                let vec_sorted: Vec<TagId> =
                    vec_orig_order
                    .clone()
                    .drain(..)
                    .collect::<std::collections::BTreeSet<_>>()
                    .iter()
                    .map(|t| t.clone())
                    .collect();

                if vec_sorted != vec_orig_order
                {
                    return Err(ParseError::new(format!("Invalid TagSet string: {:?}", val)))
                }

                Ok(TagSet(vec_sorted.iter().map(|t| t.to_db_field()).collect()))
            },
        }
    }
}

impl ToString for TagSet
{
    fn to_string(&self) -> String
    {
        self.to_db_field().unwrap_or_default()
    }
}