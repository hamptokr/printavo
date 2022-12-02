const MAX_RETRIES: u32 = 3;

pub mod auth;
pub mod error;
pub mod from_response;
pub mod orders;
pub mod page;
pub mod params;

use reqwest::header::HeaderName;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use url::Url;

use auth::Auth;
use from_response::FromResponse;

pub use page::{Page, PageMeta};

/// A convenience type with a default error type of [`Error`].
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

const PRINTAVO_BASE_URL: &str = "https://www.printavo.com";

#[derive(Clone, Debug)]
enum AuthState {
    /// No state
    None,
    /// Token Auth via Printavo's long lived tokens
    TokenAuth { email: String, token: String },
}

/// Printavo API Version to use
#[derive(Clone, Debug)]
pub enum Version {
    V1,
}

impl Default for Version {
    fn default() -> Self {
        Version::V1
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V1 => f.write_str("v1"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Printavo {
    client: reqwest::Client,
    auth_state: AuthState,
    pub base_url: Url,
    version: Version,
}

impl Printavo {
    pub fn builder() -> PrintavoBuilder {
        PrintavoBuilder::default()
    }

    /// Returns an absolute url version of `url` using the `base_url` (default:
    /// `https://printavo.com`)
    pub fn absolute_url(&self, url: impl AsRef<str>) -> Result<Url> {
        Ok(self.base_url.join(url.as_ref())?)
    }

    pub fn orders(&self) -> orders::OrdersHandler {
        orders::OrdersHandler::new(self)
    }

    /// Send a `GET` request to `route` with optional query parameters, returning
    /// the body of the response.
    pub async fn get<R, A, P>(&self, route: A, parameters: Option<&P>) -> Result<R>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
        R: FromResponse,
    {
        self.get_with_headers(route, parameters, None).await
    }

    /// Send a `GET` request to `route` with optional query parameters and headers, returning
    /// the body of the response.
    pub async fn get_with_headers<R, A, P>(
        &self,
        route: A,
        parameters: Option<&P>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<R>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
        R: FromResponse,
    {
        let response = self
            ._get_with_headers(self.absolute_url(route)?, parameters, headers)
            .await?;
        R::from_response(response).await
    }

    /// Send a `GET` request including option to set headers, with no additional post-processing.
    pub async fn _get_with_headers<P: Serialize + ?Sized>(
        &self,
        url: impl reqwest::IntoUrl,
        parameters: Option<&P>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response> {
        let mut request = self.client.get(url);

        if let Some(parameters) = parameters {
            request = request.query(parameters);
        }

        if let Some(headers) = headers {
            request = request.headers(headers)
        }

        self.execute(request).await
    }

    /// Send a `POST` request to `route` with an optional body, returning the body
    /// of the response.
    pub async fn post<P: Serialize + ?Sized, R: FromResponse>(
        &self,
        route: impl AsRef<str>,
        body: Option<&P>,
    ) -> Result<R> {
        let response = self._post(self.absolute_url(route)?, body).await?;
        R::from_response(response).await
    }

    /// Send a `POST` request with no additional pre/post-processing.
    pub async fn _post<P: Serialize + ?Sized>(
        &self,
        url: impl reqwest::IntoUrl,
        body: Option<&P>,
    ) -> Result<reqwest::Response> {
        let mut request = self.client.post(url);

        if let Some(body) = body {
            request = request.json(body);
        }

        self.execute(request).await
    }

    /// Execute the given `request` using printavo's Client.
    pub async fn execute(&self, mut request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let mut retries = 0;
        loop {
            // Saved request that we can retry later if necessary
            let mut retry_request = None;

            match self.auth_state {
                AuthState::None => (),
                AuthState::TokenAuth {
                    ref email,
                    ref token,
                } => {
                    retry_request = Some(request.try_clone().unwrap());
                    request = request.query(&[("email", email), ("token", token)]);
                }
            }

            let result = request.send().await;
            let status = match &result {
                Ok(v) => Some(v.status()),
                Err(e) => e.status(),
            };
            if let Some(StatusCode::UNAUTHORIZED) = status {
                if let Some(retry) = retry_request {
                    if retries < MAX_RETRIES {
                        retries += 1;
                        request = retry;
                        continue;
                    }
                }
            }
            return Ok(result?);
        }
    }
}

impl Default for Printavo {
    fn default() -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .user_agent("printavo-rust")
                .build()
                .unwrap(),
            auth_state: AuthState::None,
            base_url: Url::parse(PRINTAVO_BASE_URL).unwrap(),
            version: Version::V1,
        }
    }
}

#[derive(Default)]
pub struct PrintavoBuilder {
    auth: Auth,
    extra_headers: Vec<(HeaderName, String)>,
    base_url: Option<Url>,
    version: Version,
}

impl PrintavoBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Authenticate with Printavo's long lived tokens
    pub fn token_auth(mut self, email: String, token: String) -> Self {
        self.auth = Auth::Token {
            email,
            token: SecretString::new(token),
        };
        self
    }

    /// Set the base url for `Printavo`
    pub fn base_url(mut self, base_url: impl reqwest::IntoUrl) -> Result<Self> {
        self.base_url = Some(base_url.into_url()?);
        Ok(self)
    }

    /// Add an additional header to include with every request.
    pub fn add_header(mut self, key: HeaderName, value: String) -> Self {
        self.extra_headers.push((key, value));
        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn build(self) -> Result<Printavo> {
        let mut headers = reqwest::header::HeaderMap::new();

        let auth_state = match self.auth {
            Auth::None => AuthState::None,
            Auth::Token { email, token } => AuthState::TokenAuth {
                email,
                token: token.expose_secret().to_string(),
            },
        };

        for (key, value) in self.extra_headers.into_iter() {
            headers.append(key, value.parse().unwrap());
        }

        let client = reqwest::Client::builder()
            .user_agent("printavo-rust")
            .default_headers(headers)
            .build()?;

        Ok(Printavo {
            client,
            auth_state,
            base_url: self
                .base_url
                .unwrap_or_else(|| Url::parse(PRINTAVO_BASE_URL).unwrap()),
            version: self.version,
        })
    }
}
