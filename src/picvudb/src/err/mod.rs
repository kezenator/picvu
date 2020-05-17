use snafu::Snafu;
use snafu::IntoError;

#[derive(Debug, Snafu)]
pub enum Error
{
    #[snafu(display("SQLite Database Error: {:?}", source))]
    SqliteDatabaseError { source: diesel::result::Error },
    #[snafu(display("Database Consistency Error: {}", msg))]
    DatabaseConsistencyError { msg: String },
    #[snafu(display("Invalid MIME type: {:?}", source))]
    MimeError { source: mime::FromStrError },
}

impl From<diesel::result::Error> for Error
{
    fn from(source: diesel::result::Error) -> Self {
        SqliteDatabaseError{}.into_error(source)
    }
}

impl From<mime::FromStrError> for Error
{
    fn from(source: mime::FromStrError) -> Self {
        MimeError{}.into_error(source)
    }
}
