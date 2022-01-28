use snafu::Snafu;
use snafu::IntoError;

use crate::bulk::sync::SyncError;

#[derive(Debug, Snafu)]
pub enum ImportError
{
    #[snafu(display("IO Error: {:?}", source))]
    IoError { source: std::io::Error },
    #[snafu(display("DB Connection Error: {:?}", source))]
    DbConnectionError { source: picvudb::DbConnectionError },
    #[snafu(display("DB Error: {:?}", source))]
    DbError { source: picvudb::Error },
    #[snafu(display("Sync Error: {:?}", source))]
    GoogleSyncError { source: SyncError },
}

impl From<std::io::Error> for ImportError
{
    fn from(source: std::io::Error) -> Self {
        IoSnafu{}.into_error(source)
    }
}

impl From<picvudb::DbConnectionError> for ImportError
{
    fn from(source: picvudb::DbConnectionError) -> Self {
        DbConnectionSnafu{}.into_error(source)
    }
}

impl From<picvudb::Error> for ImportError
{
    fn from(source: picvudb::Error) -> Self {
        DbSnafu{}.into_error(source)
    }
}

impl From<SyncError> for ImportError
{
    fn from(source: SyncError) -> Self {
        GoogleSyncSnafu{}.into_error(source)
    }
}
