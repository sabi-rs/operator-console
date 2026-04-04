use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use tracing::Subscriber;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

const DEFAULT_FILTER: &str = "info";
const STDERR_OVERRIDE_ENV: &str = "OPERATOR_CONSOLE_LOG_STDERR";

pub fn make_tracing_subscriber<W>(writer: W) -> impl Subscriber + Send + Sync
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    tracing_subscriber::registry()
        .with(default_env_filter())
        .with(
            fmt::layer()
                .with_writer(writer)
                .with_target(true)
                .with_ansi(false)
                .without_time(),
        )
}

pub fn init_tracing() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let writer = if should_log_to_stderr() {
        BoxMakeWriter::new(io::stderr)
    } else {
        BoxMakeWriter::new(AppendFileMakeWriter::new(default_log_path()))
    };

    tracing::subscriber::set_global_default(make_tracing_subscriber(writer))
}

fn default_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER))
}

fn should_log_to_stderr() -> bool {
    env::var_os(STDERR_OVERRIDE_ENV).is_some() || !io::stderr().is_terminal()
}

fn default_log_path() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("sabi")
        .join("logs")
        .join("operator-console.log")
}

#[derive(Clone, Debug)]
struct AppendFileMakeWriter {
    path: PathBuf,
}

impl AppendFileMakeWriter {
    fn new(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        Self { path }
    }
}

impl<'writer> MakeWriter<'writer> for AppendFileMakeWriter {
    type Writer = AppendFileWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .ok();
        AppendFileWriter { file }
    }
}

struct AppendFileWriter {
    file: Option<File>,
}

impl Write for AppendFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.file.as_mut() {
            Some(file) => file.write(buf),
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.file.as_mut() {
            Some(file) => file.flush(),
            None => Ok(()),
        }
    }
}
