//! monitorbot executable.

use std::collections::HashMap;
use std::process::ExitCode;
use url::Url;

mod logging;
mod params;

use params::{Params, Parser};

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

    eprintln!("state dir: {}", &params.state_dir.display());
    for url in &params.urls {
        let url = url.clone();
        println!("Request URL: {url}");
        let res = client.get(url).send().await?;

        let actual_url = res.url().clone();
        println!("Actual URL:  {actual_url}");
        eprintln!("Response: {:?} {}", res.version(), res.status());
        eprintln!("Headers: {:#?}\n", res.headers());

        let body = res.text().await?;
        let md = render_html(&body, &actual_url);

        println!("{md}");
    }

    Ok(ExitCode::SUCCESS)
}

fn render_html(html: &str, url: &Url) -> String {
    html2md::parse_html_custom_base(
        &html,
        &HashMap::new(),
        true,
        &Some(url.clone()),
    )
    .replace("\n", "\n\n") // FIXME
}
