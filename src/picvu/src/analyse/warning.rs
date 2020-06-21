#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Warning
{
    pub kind: WarningKind,
    pub filename: String,
    pub details: String,
}

impl Warning
{
    pub fn new<S1: Into<String>, S2: Into<String>>(filename: S1, kind: WarningKind, details: S2) -> Self
    {
        Warning
        {
            kind,
            filename: filename.into(),
            details: details.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum WarningKind
{
    ImgExifDecode,
    ImgExifAnalyse,
    MvImgAnalysisError,
    VideoAnalysis,
    VideoAnalysisError,
    SkippedDuplicateMvImgPart,
    NoGoogleTakeoutMetadataAvailable,
    MissingDimensions,
    MissingDuration,
    ReverseGeocodeError,
    DuplicateGooglePhotosFilename,
    MissingGooglePhotosReference,
}
