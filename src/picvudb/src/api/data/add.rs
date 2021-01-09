use crate::api::data::{Censor, Date, Dimensions, Duration, ExternalReference, Location, NotesMarkdown, Orientation, Rating, TagKind, TitleMarkdown};

#[derive(Debug)]
pub struct Attachment
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Tag
{
    pub name: String,
    pub kind: TagKind,
    pub rating: Rating,
    pub censor: Censor,
}

#[derive(Debug)]
pub struct ObjectData
{
    pub title: Option<TitleMarkdown>,
    pub notes: Option<NotesMarkdown>,
    pub rating: Rating,
    pub censor: Censor,
    pub created_time: Option<Date>,
    pub modified_time: Option<Date>,
    pub activity_time: Option<Date>,
    pub location: Option<Location>,
    pub attachment: Attachment,
    pub tags: Vec<Tag>,
    pub ext_ref: Option<ExternalReference>,
}
