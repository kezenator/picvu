use serde::Deserialize;
use crate::analyse;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize)]
pub enum ViewObjectsListType
{
    ThumbnailsGrid,
    DetailsTable,
}

pub struct ViewObjectsList
{
    pub response: picvudb::msgs::GetObjectsResponse,
    pub list_type: ViewObjectsListType,
}

pub struct ViewSingleObject
{
    pub object: picvudb::data::get::ObjectMetadata,
    pub image_analysis: Result<Option<(analyse::img::ImgAnalysis, Vec<String>)>, analyse::img::ImgAnalysisError>,
    pub mvimg_split: analyse::img::MvImgSplit,
}
