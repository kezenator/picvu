use oauth2::{
    AuthorizationCode, AuthUrl,
    ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, ResponseType,
    Scope, StandardTokenResponse,
    TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
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
    StartedNew{client: BasicClient, csrf_token: CsrfToken, pkce_verifier: PkceCodeVerifier},
    GotCallback{client: BasicClient, code: AuthorizationCode, pkce_verifier: PkceCodeVerifier},
    Error{err: String},
    Done{access_token: AccessToken},
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

    pub fn access_token(&self) -> Option<AccessToken>
    {
        if let ClientState::Done{access_token} = &self.state
        {
            Some(access_token.clone())
        }
        else
        {
            None
        }
    }

    pub fn start_new(&mut self, setup: GoogleAuthSetup) -> String
    {
        let client = BasicClient::new(
            ClientId::new(setup.client_id),
            Some(ClientSecret::new(setup.client_secret)),
            AuthUrl::from_url(Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap()),
            Some(TokenUrl::from_url(Url::parse("https://oauth2.googleapis.com/token").unwrap())))
            .set_redirect_uri(RedirectUrl::from_url(Url::parse(&setup.redirect_url).unwrap()));

        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("https://www.googleapis.com/auth/photoslibrary".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            // Set response type and extensions
            .set_response_type(&ResponseType::new("code".to_owned()))
            .add_extra_param("access_type", "offline")
            .add_extra_param("include_granted_scopes", "true")
            .url();

        self.state = ClientState::StartedNew{ client, csrf_token, pkce_verifier };

        auth_url.to_string()
    }

    pub fn got_callback(&mut self, code: Option<String>, state: Option<String>, error: Option<String>)
    {
        if let ClientState::StartedNew{client, csrf_token, pkce_verifier} = &self.state
        {
            if let Some(code) = code
            {
                if let Some(state) = state
                {
                    if state == *csrf_token.secret()
                    {
                        let code = AuthorizationCode::new(code);

                        self.state = ClientState::GotCallback{
                            client: client.clone(),
                            code: code,
                            pkce_verifier: PkceCodeVerifier::new(pkce_verifier.secret().clone()) };
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
        if let ClientState::GotCallback{client, code, pkce_verifier} = &self.state
        {
            return ExchangeOperation::new(
                client.clone(),
                code.clone(),
                PkceCodeVerifier::new(pkce_verifier.secret().clone()));
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

    pub fn save_token(&mut self, response: GoogleAuthTokenReponse)
    {
        self.state = ClientState::Done{ access_token: AccessToken{ token: response.response.access_token().secret().clone() }};
    }
}

pub enum ExchangeOperation
{
    Continue{ client: BasicClient, code: AuthorizationCode, pkce_verifier: PkceCodeVerifier },
    Error{ err: String },
}

impl ExchangeOperation
{
    fn new(client: BasicClient, code: AuthorizationCode, pkce_verifier: PkceCodeVerifier) -> Self
    {
        ExchangeOperation::Continue{client, code, pkce_verifier}
    }

    fn error(err: String) -> Self
    {
        ExchangeOperation::Error{err}
    }

    pub fn blocking_execute(self) -> Result<GoogleAuthTokenReponse, GoogleAuthError>
    {
        match self
        {
            ExchangeOperation::Continue{client, code, pkce_verifier} =>
            {
                client.exchange_code(code)
                    .set_pkce_verifier(pkce_verifier)
                    .request(http_client)
                    .map_err(|err| GoogleAuthError::Oauth2{err: format!("{}", err)})
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
    Oauth2{err: String},
}

#[derive(Debug)]
pub struct GoogleAuthTokenReponse
{
    response: StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
}

#[derive(Clone)]
pub struct AccessToken
{
    token: String,
}

impl AccessToken
{
    pub(crate) fn secret(&self) -> String
    {
        self.token.clone()
    }
}
