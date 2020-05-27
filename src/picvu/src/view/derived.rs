use crate::analyse;

pub enum ViewObjectsListType
{
    ThumbnailsGrid,
    DetailsTable,
}

pub struct ViewObjectsList
{
    pub response: picvudb::msgs::GetObjectsResponse,
    pub view_type: ViewObjectsListType,
}

pub struct ViewSingleObject
{
    pub object: picvudb::data::get::ObjectMetadata,
    pub image_analysis: Result<Option<(analyse::img::ImgAnalysis, Vec<String>)>, analyse::img::ImgAnalysisError>,
}
