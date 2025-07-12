//! monitorbot executable.

use bytes::Bytes;
use encoding_rs::Encoding;
use htmd::HtmlToMarkdown;
use mime::Mime;
use std::borrow::Cow;
use std::collections::vec_deque::VecDeque;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::process::ExitCode;
use termcolor::{Color, ColorSpec};
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
        Ok(self.content_type()?.and_then(|media_type| {
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

    let state_dir_path = params.state_dir_path();
    fs::DirBuilder::new()
        .recursive(true)
        .create(&state_dir_path)?;

    for request_url in &params.urls {
        let mut file_name = fs_safe_url(request_url);
        file_name.push_str(".ron");
        let request_path = state_dir_path.join(file_name);

        let old_response: Option<Response> = if request_path.exists() {
            Some(ron::de::from_bytes(&std::fs::read(&request_path)?)?)
        } else {
            None
        };

        // FIXME use etag/last-modified to check if possible.
        let response = Response::from_reqwest(
            client.get(request_url.clone()).send().await?,
        )
        .await?;

        let mut response_file_name = fs_safe_url(&response.url);
        response_file_name.push_str(".ron");
        let response_path = state_dir_path.join(&response_file_name);

        // FIXME: atomic write
        std::fs::write(
            &response_path,
            ron::ser::to_string_pretty(
                &response,
                ron::ser::PrettyConfig::default(),
            )?,
        )?;

        if response.url != *request_url {
            // FIXME do this for any other steps in the redirect chain.

            // FIXME make this atomic
            if request_path.exists() {
                fs::remove_file(&request_path)?;
            }

            // They’re in the same directory, so just link to the file name.
            symlink(&response_file_name, &request_path)?;
        }

        let old_md = if let Some(old_response) = old_response {
            // Shortcut
            if old_response.body == response.body {
                continue;
            }

            // FIXME check the content-type; handle non-HTML.
            render_html(&old_response.text()?, &old_response.url)?
        } else {
            String::new()
        };

        // FIXME check the content-type; handle non-HTML.
        let new_md = render_html(&response.text()?, &response.url)?;
        if params.no_diff {
            println!("{new_md}");
        } else if new_md != old_md {
            print_pretty_diff(&mut params.out_stream(), &old_md, &new_md);
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// Make a filesystem-safe version of the URL.
fn fs_safe_url(url: &Url) -> String {
    // FIXME does not work on Windows.
    let s = url.as_str();
    assert_ne!(s, "");
    assert_ne!(s, ".");
    assert_ne!(s, "..");
    s.replace('\\', "\\\\")
        .replace('|', r"\|")
        .replace('/', "|")
}

/// Render HTML as Markdown.
fn render_html<S: AsRef<str>>(
    html: S,
    _base_url: &Url,
) -> anyhow::Result<String> {
    // FIXME output links relative to _base_url.
    Ok(HtmlToMarkdown::builder().build().convert(html.as_ref())?)
}

/// Print a pretty diff.
#[allow(clippy::iter_with_drain)] // Lint is incorrect
fn print_pretty_diff<S>(out: &mut S, old: &str, new: &str)
where
    S: termcolor::WriteColor + io::Write,
{
    const CONTEXT_LEN: usize = 2;

    let mut context = VecDeque::new();
    let mut lines_since_diff: Option<usize> = None;

    let mut old_color = ColorSpec::new();
    old_color.set_fg(Some(Color::Red)).set_intense(true);
    let mut new_color = ColorSpec::new();
    new_color.set_fg(Some(Color::Green)).set_intense(true);

    for diff in diff::lines(old, new) {
        match diff {
            diff::Result::Left(old_line) => {
                for line in context.drain(..) {
                    println!(" {line}");
                }
                // Use `unwrap()` here because these would be IO errors, so we
                // may as well act like `println!`.
                out.set_color(&old_color).unwrap();
                writeln!(out, "-{old_line}").unwrap();
                out.reset().unwrap();
                lines_since_diff = Some(0);
            }
            diff::Result::Right(new_line) => {
                for line in context.drain(..) {
                    println!(" {line}");
                }
                // Use `unwrap()` here because these would be IO errors, so we
                // may as well act like `println!`.
                out.set_color(&new_color).unwrap();
                writeln!(out, "+{new_line}").unwrap();
                out.reset().unwrap();
                lines_since_diff = Some(0);
            }
            diff::Result::Both(line, _) => {
                if let Some(count) = lines_since_diff {
                    println!(" {line}");
                    #[allow(clippy::arithmetic_side_effects)]
                    let count = count + 1;
                    if count >= CONTEXT_LEN {
                        lines_since_diff = None;
                    } else {
                        lines_since_diff = Some(count);
                    }
                } else {
                    context.push_back(line);
                    if context.len() > CONTEXT_LEN {
                        context.pop_front();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::check;

    /// Convert a `&str` to a `Url`.
    fn u(s: &'static str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn test_fs_safe_url() {
        check!(
            fs_safe_url(&u("https://demon.horse/hireme/#fragment"))
                == "https:||demon.horse|hireme|#fragment"
        );
        check!(fs_safe_url(&u("a://a/b")) == "a:||a|b");
        check!(
            fs_safe_url(&u(r"a://a/foo\back|pipe\|backpipe"))
                == r"a:||a|foo\\back\|pipe\\\|backpipe"
        );
    }
}
