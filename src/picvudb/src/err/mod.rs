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
    #[snafu(display("Data parse error: {:?}", source))]
    DataParseError { source: ParseError },
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

impl From<ParseError> for Error
{
    fn from(source: ParseError) -> Self {
        DataParseError{}.into_error(source)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError(String);

impl ParseError
{
    pub fn new<T: Into<String>>(s: T) -> Self
    {
        ParseError(s.into())
    }
}

impl std::fmt::Display for ParseError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(f, "Parse Error: {}", self.0)
    }
}

impl std::error::Error for ParseError
{
}
