mod book;
mod cat;
mod error;
mod extract;
mod meta;

use clap::{Parser, Subcommand};
use epub::doc::EpubDoc;
use std::path::{Path, PathBuf};

use crate::cat::cat;
use crate::error::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract metadata from an ePub file
    Meta {
        /// Path to the ePub file
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Dump ePub contents as raw text
    Raw {
        /// Path to the ePub file
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Dump ePub contents as JSON.
    Json {
        /// Path to the ePub file
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
}

/// Extract metadata from an ePub file
fn meta<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let book = EpubDoc::new(path)?;
    let stdout = ::std::io::stdout();
    let handle = stdout.lock();
    let map = meta::meta_vars_from_metadata(&book);
    serde_json::to_writer(handle, &map).map_err(Error::JsonError)
}

/// Dump ePub contents as JSON
fn json<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let mut book = EpubDoc::new(path)?;
    cat(&mut book, true)
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Meta { path } => meta(path)?,
        Commands::Json { path } => json(path)?,
        Commands::Raw { path } => cat(&mut EpubDoc::new(path)?, false)?,
    }
    Ok(())
}
