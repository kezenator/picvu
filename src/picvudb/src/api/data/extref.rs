use crate::ParseError;

#[derive(Debug, Clone)]
pub enum ExternalReference
{
    GooglePhotos{ id: String },
}

impl ExternalReference
{
    pub fn get_type(&self) -> String
    {
        match self
        {
            Self::GooglePhotos{..} => "Google Photos".to_owned(),
        }
    }

    pub fn get_id(&self) -> String
    {
        match self
        {
            Self::GooglePhotos{id} => id.clone(),
        }
    }

    pub fn get_url(&self) -> String
    {
        match self
        {
            Self::GooglePhotos{id} => format!("https://photos.google.com/lr/photo/{}", id),
        }
    }

    pub(crate) fn from_db_fields(ref_type: Option<String>, ref_id: Option<String>) -> Result<Option<Self>, ParseError>
    {
        match (ref_type.clone(), ref_id.clone())
        {
            (None, None) =>
            {
                return Ok(None);
            },
            (Some(ref_type), Some(ref_id)) =>
            {
                if ref_type == "gp"
                {
                    return Ok(Some(ExternalReference::GooglePhotos{id: ref_id}));
                }
            }
            _ =>
            {
            },
        }

        Err(ParseError::new(format!("Invalid external reference [{:?}, {:?}]", ref_type, ref_id)))
    }

    pub(crate) fn to_db_field_type(&self) -> String
    {
        match self
        {
            Self::GooglePhotos{..} => "gp".to_owned(),
        }
    }

    pub(crate) fn to_db_field_id(&self) -> String
    {
        match self
        {
            Self::GooglePhotos{id} => id.clone(),
        }
    }
}
