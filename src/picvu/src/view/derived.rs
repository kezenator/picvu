use crate::analyse;

pub struct ViewObjectDetails
{
    pub object: picvudb::data::get::ObjectMetadata,
    pub image_analysis: Result<Option<analyse::img::ImgAnalysis>, analyse::img::ImgAnalysisError>,
}
