//! monitorbot executable.

use bytes::Bytes;
use encoding_rs::Encoding;
use mime::Mime;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::process::ExitCode;
use thiserror::Error;
use url::Url;

mod logging;
mod params;

use params::{Params, Parser};

/// Default user agent to use when making HTTP requests.
static USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Wrapper to handle errors.
///
/// See [`cli()`].
#[tokio::main]
async fn main() -> ExitCode {
    let params = Params::parse();
    cli(&params).await.unwrap_or_else(|error| {
        params.warn(format!("Error: {error:#}\n")).unwrap();
        ExitCode::FAILURE
    })
}

/// Errors resulting from processing an HTTP response.
#[derive(Error, Debug)]
pub enum ResponseError {
    /// Could not convert header value to string.
    #[error("could not convert header value to string")]
    InvalidStr(#[from] http::header::ToStrError),

    /// Could not convert header value to MIME media type.
    #[error("could not convert header value to MIME media type")]
    InvalidMediaType(#[from] mime::FromStrError),

    /// Unknown charset in header.
    #[error("unknown charset {0}")]
    InvalidCharset(String),
}

/// An HTTP response that can be serialized.
#[derive(serde::Serialize, serde::Deserialize)]
struct Response {
    /// The URL that actually produced the response.
    pub url: Url,

    /// The HTTP version of the response.
    #[serde(with = "http_serde::version")]
    pub version: http::Version,

    /// The HTTP status code of the response.
    #[serde(with = "http_serde::status_code")]
    pub status: http::StatusCode,

    /// The HTTP headers of the response.
    #[serde(with = "http_serde::header_map")]
    pub headers: http::HeaderMap,

    /// The body returned by the response.
    pub body: Bytes,
}

impl Response {
    /// From [`reqwest::Response`].
    pub async fn from_reqwest(
        response: reqwest::Response,
    ) -> reqwest::Result<Self> {
        Ok(Self {
            url: response.url().clone(),
            version: response.version(),
            status: response.status(),
            headers: response.headers().clone(),
            body: response.bytes().await?,
        })
    }

    /// Get the content-type.
    ///
    /// Based on [`reqwest::Response::text_with_charset()`].
    pub fn content_type(&self) -> Result<Option<Mime>, ResponseError> {
        // FIXME? ignores multiple values
        self.headers
            .get(http::header::CONTENT_TYPE)
            .map(|value| {
                value.to_str().map_err(ResponseError::InvalidStr).and_then(
                    |value| {
                        value.parse().map_err(ResponseError::InvalidMediaType)
                    },
                )
            })
            .transpose()
    }

    /// Get the charset.
    ///
    /// Based on [`reqwest::Response::text_with_charset()`].
    pub fn charset(&self) -> Result<Option<String>, ResponseError> {
        // FIXME? return &str?
        Ok(self
            .content_type()?
            .and_then(|media_type| {
                media_type
                    .get_param(mime::CHARSET)
                    .map(|name| name.to_string())
            }))
    }

    /// Get the charset.
    ///
    /// Based on [`reqwest::Response::text_with_charset()`].
    pub fn charset_encoding(
        &self,
    ) -> Result<Option<&'static Encoding>, ResponseError> {
        self.charset()?
            .map(|charset| {
                Encoding::for_label(charset.as_bytes())
                    .ok_or(ResponseError::InvalidCharset(charset))
            })
            .transpose()
    }

    /// Get the response body as text.
    ///
    /// If the response does not specify a charset, this defaults to UTF-8.
    pub fn text(&self) -> Result<Cow<'_, str>, ResponseError> {
        // FIXME change fallback to Windows Latin 1?
        let encoding = self.charset_encoding()?.unwrap_or(encoding_rs::UTF_8);
        let (text, _actual_encoding, _mangled) = encoding.decode(&self.body);
        Ok(text)
    }
}

/// Do the actual work.
///
/// Returns the exit code to use.
///
/// # Errors
///
/// This returns any errors encountered during the run so that they can be
/// outputted nicely in [`main()`].
async fn cli(params: &Params) -> anyhow::Result<ExitCode> {
    logging::init(params.verbose)?;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connection_verbose(true)
        .build()?;

    fs::DirBuilder::new()
        .recursive(true)
        .create(params.state_dir_path())?;

    for url in &params.urls {
        let url = url.clone();
        println!("Request URL: {url}");

        let response =
            Response::from_reqwest(client.get(url).send().await?).await?;
        println!("Actual URL:  {}", &response.url);
        eprintln!("Response: {:?} {}", response.version, response.status);
        eprintln!("Headers: {:#?}\n", response.headers);

        // FIXME check the content-type; handle non-HTML.
        let body = response.text()?;
        let md = render_html(&body, &response.url);
        println!("{md}");
    }

    Ok(ExitCode::SUCCESS)
}

/// Render HTML as Markdown.
fn render_html<S: AsRef<str>>(html: S, url: &Url) -> String {
    html2md::parse_html_custom_base(
        html.as_ref(),
        &HashMap::new(),
        true,
        &Some(url.clone()),
    )
    .replace('\n', "\n\n") // FIXME
}
