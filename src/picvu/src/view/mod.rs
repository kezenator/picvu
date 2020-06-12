use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::dev::HttpResponseBuilder;
use actix_web::ResponseError;

mod doc;

pub use doc::redirect;
pub use doc::err;
pub use doc::html_page;
pub use doc::binary;

#[derive(Debug)]
pub enum ErrorResponder
{
    ActixMailboxError(actix::MailboxError),
    PicvudbError(picvudb::Error),
    PicvudbParseError(picvudb::ParseError),
    MultipartError(actix_multipart::MultipartError),
    StdIoError(std::io::Error),
    BlockingOperationCanceled,
    ImageError(image::error::ImageError),
    GoogleAuthError(googlephotos::auth::GoogleAuthError),
    GoogleTimezoneError(googlephotos::timezone::TimezoneError),
    GoogleGeocodeError(googlephotos::geocode::GeocodeError),
}

impl std::fmt::Display for ErrorResponder
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{:?}", self)
    }
}

impl From<actix::MailboxError> for ErrorResponder
{
    fn from(error: actix::MailboxError) -> Self
    {
        ErrorResponder::ActixMailboxError(error)
    }
}

impl From<picvudb::Error> for ErrorResponder
{
    fn from(error: picvudb::Error) -> Self
    {
        ErrorResponder::PicvudbError(error)
    }
}

impl From<picvudb::ParseError> for ErrorResponder
{
    fn from(error: picvudb::ParseError) -> Self
    {
        ErrorResponder::PicvudbParseError(error)
    }
}

impl From<actix_multipart::MultipartError> for ErrorResponder
{
    fn from(error: actix_multipart::MultipartError) -> Self
    {
        ErrorResponder::MultipartError(error)
    }
}

impl From<std::io::Error> for ErrorResponder
{
    fn from(error: std::io::Error) -> Self
    {
        ErrorResponder::StdIoError(error)
    }
}

impl<T> From<actix_web::error::BlockingError<T>> for ErrorResponder
    where T: Into<ErrorResponder> + Debug
{
    fn from(error: actix_web::error::BlockingError<T>) -> Self
    {
        match error
        {
            actix_web::error::BlockingError::Canceled => ErrorResponder::BlockingOperationCanceled,
            actix_web::error::BlockingError::Error(e) => e.into(),
        }
    }
}

impl From<image::error::ImageError> for ErrorResponder
{
    fn from(error: image::error::ImageError) -> Self
    {
        ErrorResponder::ImageError(error)
    }
}

impl From<googlephotos::auth::GoogleAuthError> for ErrorResponder
{
    fn from(error: googlephotos::auth::GoogleAuthError) -> Self
    {
        ErrorResponder::GoogleAuthError(error)
    }
}

impl From<googlephotos::timezone::TimezoneError> for ErrorResponder
{
    fn from(error: googlephotos::timezone::TimezoneError) -> Self
    {
        ErrorResponder::GoogleTimezoneError(error)
    }
}

impl From<googlephotos::geocode::GeocodeError> for ErrorResponder
{
    fn from(error: googlephotos::geocode::GeocodeError) -> Self
    {
        ErrorResponder::GoogleGeocodeError(error)
    }
}

impl ResponseError for ErrorResponder
{
    fn error_response(&self) -> HttpResponse
    {
        let builder = HttpResponseBuilder::new(self.status_code());
        let contents = format!("{:#?}", self);

        doc::err(builder, contents)
    }

    fn status_code(&self) -> StatusCode
    {
        match self
        {
            Self::ActixMailboxError(_)
                | Self::PicvudbError(_)
                | Self::StdIoError(_) 
                | Self::BlockingOperationCanceled
                | Self::ImageError(_) 
                | Self::GoogleAuthError(_)
                | Self::GoogleTimezoneError(_)
                | Self::GoogleGeocodeError(_) =>
            {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Self::MultipartError(_)
                | Self::PicvudbParseError(_) =>
            {
                StatusCode::BAD_REQUEST
            }
        }
    }
}
