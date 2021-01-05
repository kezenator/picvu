use crate::api::data::{Censor, Date, DateRange, Dimensions, Duration, ExternalReference, Location, NotesMarkdown, ObjectId, Orientation, Rating, TagId, TagKind, TitleMarkdown};

#[derive(Debug, Clone)]
pub struct AttachmentMetadata
{
    pub filename: String,
    pub created: Date,
    pub modified: Date,
    pub mime: mime::Mime,
    pub size: u64,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct TagMetadata
{
    pub tag_id: TagId,
    pub name: String,
    pub kind: TagKind,
    pub rating: Option<Rating>,
    pub censor: Censor,
}

#[derive(Debug, Clone)]
pub struct ObjectMetadata
{
    pub id: ObjectId,
    pub created_time: Date,
    pub modified_time: Date,
    pub activity_time: Date,
    pub title: Option<TitleMarkdown>,
    pub notes: Option<NotesMarkdown>,
    pub rating: Option<Rating>,
    pub censor: Censor,
    pub location: Option<Location>,
    pub attachment: AttachmentMetadata,
    pub tags: Vec<TagMetadata>,
    pub ext_ref: Option<ExternalReference>,
}

#[derive(Debug, Clone)]
pub struct PaginationRequest
{
    pub offset: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone)]
pub struct PaginationResponse
{
    pub offset: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub enum GetObjectsQuery
{
    ByActivityDesc,
    ByModifiedDesc,
    ByAttachmentSizeDesc,
    ByObjectId(ObjectId),
    NearLocationByActivityDesc{ location: Location, radius_meters: f64 },
    TitleNotesSearchByActivityDesc{ search: SearchString },
    TagByActivityDesc{ tag_id: TagId },
    ActivityDateRangeByActivityDesc{ date_range: DateRange },
}

#[derive(Debug, Clone)]
pub enum SearchString
{
    FullSearch(String),
    Suggestion(String),
}

impl SearchString
{
    pub fn to_fts5_query(&self) -> String
    {
        let (mut fts5_search, mut prefix) = match self.clone()
        {
            SearchString::FullSearch(search) => (search, false),
            SearchString::Suggestion(search) => (search, true),
        };

        if fts5_search.len() < 3
        {
            // Don't allow short suggestions
            prefix = false;
        }
    
        fts5_search = fts5_search.replace('\"', "\"\"");
        fts5_search = format!("\"{}\"", fts5_search);
    
        if prefix
        {
            fts5_search.push('*');
        }
    
        fts5_search
    }
    
    pub fn to_literal_string(&self) -> String
    {
        match self.clone()
        {
            SearchString::FullSearch(search) => search,
            SearchString::Suggestion(search) => search,
        }
    }
}
