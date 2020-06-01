use oauth2::prelude::*;
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, RequestTokenError, ResponseType,
    Scope, StandardTokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use url::Url;

pub struct GoogleAuthSetup
{
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

enum ClientState
{
    None,
    StartedNew{client: BasicClient, csrf_token: CsrfToken},
    GotCallback{client: BasicClient, code: AuthorizationCode},
    Error{err: String},
}

pub struct GoogleAuthClient
{
    state: ClientState,
}

impl GoogleAuthClient
{
    pub fn new() -> Self
    {
        GoogleAuthClient
        {
            state: ClientState::None,
        }
    }

    pub fn start_new(&mut self, setup: GoogleAuthSetup) -> String
    {
        let client = BasicClient::new(
            ClientId::new(setup.client_id),
            Some(ClientSecret::new(setup.client_secret)),
            AuthUrl::new(Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap()),
            Some(TokenUrl::new(Url::parse("https://oauth2.googleapis.com/token").unwrap())))
            .add_scope(Scope::new("https://www.googleapis.com/auth/photoslibrary".to_string()))
            .set_redirect_url(RedirectUrl::new(Url::parse(&setup.redirect_url).unwrap()));

        let response_type = ResponseType::new("code".to_owned());
        let extensions = [("access_type", "offline"), ("include_granted_scopes", "true")];

        let (auth_url, csrf_token) = client.authorize_url_extension(&response_type, CsrfToken::new_random, &extensions);

        self.state = ClientState::StartedNew{ client, csrf_token };

        auth_url.to_string()
    }

    pub fn got_callback(&mut self, code: Option<String>, state: Option<String>, error: Option<String>)
    {
        if let ClientState::StartedNew{client, csrf_token} = &self.state
        {
            if let Some(code) = code
            {
                if let Some(state) = state
                {
                    if state == *csrf_token.secret()
                    {
                        let code = AuthorizationCode::new(code);

                        self.state = ClientState::GotCallback{ client: client.clone(), code };
                    }
                    else
                    {
                        self.state = ClientState::Error{ err: "The authentication callback state doesn't match".to_owned() };
                    }
                }
                else
                {
                    self.state = ClientState::Error{ err: "No state was provided by the authentication server".to_owned() };
                }
            }
            else
            {
                self.state = ClientState::Error{ err: error.unwrap_or("Unknown error".to_owned()) };
            }
        }
    }

    pub fn exchange_token(&mut self) -> ExchangeOperation
    {
        if let ClientState::GotCallback{client, code} = &self.state
        {
            return ExchangeOperation::new(client.clone(), code.clone());
        }
        else if let ClientState::Error{err} = &self.state
        {
            return ExchangeOperation::error(err.clone());
        }
        else
        {
            return ExchangeOperation::error("Invalid progression - try again".to_owned());
        }
    }
}

pub enum ExchangeOperation
{
    Continue{ client: BasicClient, code: AuthorizationCode },
    Error{ err: String },
}

impl ExchangeOperation
{
    fn new(client: BasicClient, code: AuthorizationCode) -> Self
    {
        ExchangeOperation::Continue{client, code}
    }

    fn error(err: String) -> Self
    {
        ExchangeOperation::Error{err}
    }

    pub fn blocking_execute(self) -> Result<GoogleAuthTokenReponse, GoogleAuthError>
    {
        match self
        {
            ExchangeOperation::Continue{client, code} =>
            {
                client.exchange_code(code)
                    .map_err(|err| GoogleAuthError::Oauth2{err})
                    .map(|response| GoogleAuthTokenReponse{response})
            },
            ExchangeOperation::Error{err} =>
            {
                Err(GoogleAuthError::Other{err})
            },
        }
    }
}

#[derive(Debug)]
pub enum GoogleAuthError
{
    Other{err: String},
    Oauth2{err: RequestTokenError<oauth2::basic::BasicErrorResponseType>},
}

#[derive(Debug)]
pub struct GoogleAuthTokenReponse
{
    response: StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
}
