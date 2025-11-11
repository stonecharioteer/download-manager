use crate::download::utils;
use crate::download::{download_file_async, download_file_blocking, download_range_async};
use anyhow::bail;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use url::Url;

/// Download manager application.
#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// URL to a file to download
    url: Url,

    /// Target directory
    #[arg(short, long, default_value = ".download")]
    target_directory: PathBuf,

    /// Download chunk size
    #[arg(short, long, default_value_t = 65_536)]
    chunk_size: usize,

    /// Resume if the file already exists and isn't complete
    #[arg(short, long)]
    resume: bool,

    /// Overwrite existing file
    #[arg(short, long)]
    overwrite: bool,
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        self.command
            .execute(
                self.url,
                &self.target_directory,
                self.chunk_size,
                self.resume,
                self.overwrite,
            )
            .await
    }
}

#[derive(Subcommand)]
pub enum Commands {
    DownloadBlocking,
    DownloadAsync {
        /// Downloads from this byte
        #[arg(short = 's', long)]
        range_start: Option<usize>,

        /// Downloads till this byte
        #[arg(short = 'e', long)]
        range_end: Option<usize>,
    },
}

impl Commands {
    async fn execute(
        &self,
        url: Url,
        target_directory: &PathBuf,
        chunk_size: usize,
        resume: bool,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(target_directory)?;
        let bar = indicatif::ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_message(format!("Attempting to download {}", url));
        let interrupted = Arc::new(AtomicBool::new(false));
        let interrupted_clone = interrupted.clone();
        ctrlc::set_handler(move || {
            interrupted_clone.store(true, Ordering::SeqCst);
        })
        .expect("Could not set keyboard interrupt handler.");
        match &self {
            Commands::DownloadBlocking => {
                let target_directory = target_directory.clone();
                let url = url.clone();
                tokio::task::spawn_blocking(move || {
                    let path = download_file_blocking(
                        url,
                        &target_directory,
                        chunk_size,
                        resume,
                        overwrite,
                        bar,
                        interrupted,
                    )?;
                    let hash = utils::hash_file(&path, chunk_size)?;
                    println!(
                        "File downloaded to {}; SHA256: {}.",
                        path.display(),
                        hex::encode(hash)
                    );
                    Ok(())
                })
                .await?
            }
            Commands::DownloadAsync {
                range_start,
                range_end,
            } => match (range_start, range_end) {
                (None, None) => {
                    let path = download_file_async(
                        url,
                        target_directory,
                        resume,
                        overwrite,
                        bar,
                        interrupted,
                    )
                    .await?;
                    let hash = utils::hash_file(&path, chunk_size)?;
                    println!(
                        "File downloaded to {}; SHA256: {}.",
                        path.display(),
                        hex::encode(hash)
                    );
                    Ok(())
                }
                (Some(start), Some(end)) => {
                    let path =
                        download_range_async(url, target_directory, *start, *end, bar, interrupted)
                            .await?;
                    println!("File downloaded to {}.", path.display());
                    Ok(())
                }
                _ => {
                    bail!(
                        "Both --range-start and --range-end are required. You cannot pass only one.",
                    );
                }
            },
        }
    }
}
