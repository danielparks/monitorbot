//! Code to deal with executable parameters.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub use clap::Parser;

/// Parameters to configure executable.
#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub struct Params {
    /// URLs to check to changes
    pub urls: Vec<url::Url>,

    /// Whether or not to just render the page ignoring changes
    #[clap(long)]
    pub no_diff: bool,

    /// Where to store state (default: ~/.monitorbot)
    #[clap(short, long, value_hint=clap::ValueHint::DirPath)]
    pub state_dir: Option<PathBuf>,

    /// Whether or not to output in color
    #[clap(long, default_value = "auto", value_name = "WHEN")]
    pub color: ColorChoice,

    /// Verbosity (may be repeated up to three times)
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Params {
    /// Print a warning message in error color to `err_stream()`.
    pub fn warn<S: AsRef<str>>(&self, message: S) -> io::Result<()> {
        let mut err_out = self.err_stream();
        err_out.set_color(&error_color())?;
        err_out.write_all(message.as_ref().as_bytes())?;
        err_out.reset()?;

        Ok(())
    }

    /// Get stream to use for standard output.
    pub fn out_stream(&self) -> StandardStream {
        StandardStream::stdout(self.color_choice(&io::stdout()))
    }

    /// Get stream to use for errors.
    pub fn err_stream(&self) -> StandardStream {
        StandardStream::stderr(self.color_choice(&io::stderr()))
    }

    /// Whether or not to output on a stream in color.
    ///
    /// Checks if passed stream is a terminal.
    pub fn color_choice<T: IsTerminal>(
        &self,
        stream: &T,
    ) -> termcolor::ColorChoice {
        if self.color == ColorChoice::Auto && !stream.is_terminal() {
            termcolor::ColorChoice::Never
        } else {
            self.color.into()
        }
    }

    /// Get the directory to store state in.
    ///
    /// Clap’s `default_value` functionality doesn’t support dynamic values.
    pub fn state_dir_path(&self) -> PathBuf {
        self.state_dir.clone().unwrap_or_else(|| {
            std::env::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".monitorbot")
        })
    }
}

/// Whether or not to output in color
#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum ColorChoice {
    /// Output in color when running in a terminal that supports it
    Auto,

    /// Always output in color
    Always,

    /// Never output in color
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<ColorChoice> for termcolor::ColorChoice {
    fn from(choice: ColorChoice) -> Self {
        match choice {
            ColorChoice::Auto => Self::Auto,
            ColorChoice::Always => Self::Always,
            ColorChoice::Never => Self::Never,
        }
    }
}

/// Returns color used to output errors.
pub fn error_color() -> ColorSpec {
    let mut color = ColorSpec::new();
    color.set_fg(Some(Color::Red));
    color.set_intense(true);
    color
}
