use snafu::Snafu;
use snafu::IntoError;

#[derive(Debug, Snafu)]
pub enum GoogleApiError
{
    #[snafu(display("curl error: {:?}", source))]
    CurlError{ source: curl::Error },
    #[snafu(display("JSON error: {:?}", source))]
    JsonError{ source: serde_json::error::Error },
    #[snafu(display("Unexpedted response: {:?}", body))]
    UnexpectedResponse{ body: String },
}

impl GoogleApiError
{
    pub fn new_unexpected_response(body: String) -> GoogleApiError
    {
        UnexpectedResponse{body}.fail::<()>().err().unwrap()
    }
}

impl From<curl::Error> for GoogleApiError
{
    fn from(source: curl::Error) -> Self
    {
        CurlError{}.into_error(source)
    }
}

impl From<serde_json::error::Error> for GoogleApiError
{
    fn from(source: serde_json::error::Error) -> Self
    {
        JsonError{}.into_error(source)
    }
}
