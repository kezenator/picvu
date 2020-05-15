use snafu::Snafu;
use snafu::IntoError;

#[derive(Debug, Snafu)]
pub enum Error
{
    #[snafu(display("Database error: {}", source))]
    DatabaseError { source: diesel::result::Error },
}

impl From<diesel::result::Error> for Error
{
    fn from(source: diesel::result::Error) -> Self {
        DatabaseError{}.into_error(source)
    }
}
