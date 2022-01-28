use snafu::Snafu;
use snafu::IntoError;

#[derive(Debug, Snafu)]
pub enum SyncError
{
    #[snafu(display("DB Connection Error: {:?}", source))]
    DbConnectionError { source: picvudb::DbConnectionError },
    #[snafu(display("DB Error: {:?}", source))]
    DbError { source: picvudb::Error },
    #[snafu(display("Google Photos Error: {:?}", source))]
    GooglePhotosError { source: googlephotos::api::GoogleApiError },
    #[snafu(display("Google Photos Parse Error: {}", msg))]
    GooglePhotosParseError { msg: String },
}

impl SyncError
{
    pub fn new_parse_err(msg: String) -> SyncError
    {
        GooglePhotosParseSnafu{msg}.fail::<()>().err().unwrap()
    }
}

impl From<picvudb::DbConnectionError> for SyncError
{
    fn from(source: picvudb::DbConnectionError) -> Self {
        DbConnectionSnafu{}.into_error(source)
    }
}

impl From<picvudb::Error> for SyncError
{
    fn from(source: picvudb::Error) -> Self {
        DbSnafu{}.into_error(source)
    }
}

impl From<googlephotos::api::GoogleApiError> for SyncError
{
    fn from(source: googlephotos::api::GoogleApiError) -> Self {
        GooglePhotosSnafu{}.into_error(source)
    }
}
