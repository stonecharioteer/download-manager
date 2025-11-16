use crate::download::progress::DownloadProgress;
use crate::download::utils;
use crate::download::{download_file_async, download_file_blocking, download_with_workers};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
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

    /// Don't cleanup part files after merging (for debugging)
    #[arg(long)]
    no_cleanup: bool,
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
                self.no_cleanup,
            )
            .await
    }
}

#[derive(Subcommand)]
pub enum Commands {
    DownloadBlocking,
    DownloadAsync {
        /// Use workers to download, by default this is 1, for single-worker driven.
        #[arg(short, long, default_value_t = 1)]
        workers: u8,
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
        no_cleanup: bool,
    ) -> anyhow::Result<()> {
        use std::sync::atomic::Ordering;
        fs::create_dir_all(target_directory)?;

        // Print initial info
        println!("Downloading {} to {}", url, target_directory.display());
        if resume {
            println!("Resume mode enabled");
        }
        if overwrite {
            println!("Overwrite mode enabled");
        }

        let bar = indicatif::ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_message("Starting download...");
        let progress = DownloadProgress::new();
        let download_start = std::time::Instant::now();
        let interrupted_clone = progress.interrupted.clone();
        ctrlc::set_handler(move || {
            interrupted_clone.store(true, Ordering::SeqCst);
        })
        .expect("Could not set keyboard interrupt handler.");
        let progress_clone = progress.clone();
        let bar_clone = bar.clone();
        let start_time = std::time::Instant::now();
        let progress_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                let downloaded = progress_clone.bytes_downloaded.load(Ordering::Relaxed);
                let total = progress_clone.total_bytes.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs().max(1);
                let speed = downloaded as u64 / elapsed;

                if total > 0 {
                    bar_clone.set_message(format!(
                        "Downloaded: {}/{} @ {}/s",
                        indicatif::HumanBytes(downloaded as u64),
                        indicatif::HumanBytes(total),
                        indicatif::HumanBytes(speed)
                    ));
                } else {
                    bar_clone.set_message(format!(
                        "Downloaded: {} @ {}/s",
                        indicatif::HumanBytes(downloaded as u64),
                        indicatif::HumanBytes(speed)
                    ));
                }

                if progress_clone.interrupted.load(Ordering::Relaxed) {
                    bar_clone.abandon_with_message("Download interrupted.");
                    break;
                }
            }
        });
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
                        progress,
                    )?;
                    let download_time = download_start.elapsed();
                    progress_task.abort();
                    bar.finish_with_message("Download complete, hashing now.");
                    let hash = utils::hash_file(&path, chunk_size)?;
                    println!(
                        "Downloaded to {} in {}",
                        path.display(),
                        indicatif::HumanDuration(download_time)
                    );
                    println!("SHA256: {}", hex::encode(hash));
                    Ok(())
                })
                .await?
            }
            Commands::DownloadAsync { workers } => {
                if *workers <= 1 {
                    let path =
                        download_file_async(url, target_directory, resume, overwrite, progress)
                            .await?;
                    let download_time = download_start.elapsed();
                    progress_task.abort();
                    bar.finish_with_message("Download complete, hashing now.");
                    let hash = utils::hash_file(&path, chunk_size)?;
                    println!(
                        "Downloaded to {} in {}",
                        path.display(),
                        indicatif::HumanDuration(download_time)
                    );
                    println!("SHA256: {}", hex::encode(hash));
                    Ok(())
                } else {
                    let path = download_with_workers(
                        url,
                        target_directory,
                        *workers,
                        progress,
                        no_cleanup,
                    )
                    .await?;
                    let download_time = download_start.elapsed();
                    progress_task.abort();
                    bar.finish_with_message("Download complete, hashing now.");
                    let hash = utils::hash_file(&path, chunk_size)?;
                    println!(
                        "Downloaded to {} in {}",
                        path.display(),
                        indicatif::HumanDuration(download_time)
                    );
                    println!("SHA256: {}", hex::encode(hash));
                    Ok(())
                }
            }
        }
    }
}
