use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use hex;
use indicatif::{HumanBytes, HumanDuration, ProgressBar};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use std::time::Instant;

/// Download manager application.
#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// URL to a file to download
    url: String,

    /// Target directory
    #[arg(short, long, default_value = ".download")]
    target_directory: String,

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

#[derive(Subcommand)]
enum Commands {
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

fn download_file_blocking(
    url: String,
    target_dir: &Path,
    chunk_size: usize,
    resume: bool,
    overwrite: bool,
    bar: ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> Result<()> {
    let fname = url.split('/').last().unwrap_or("tmp.bin");
    let fname = target_dir.join(fname);
    println!("File to download: '{}'.", fname.to_str().unwrap());
    let mut resume_from = 0;
    let mut hasher = Sha256::new();
    let mut dest = if fname.exists() && fname.is_file() {
        if overwrite {
            let message = format!("File exists at: '{}' overwriting.", fname.to_str().unwrap());
            println!("{}", message);
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(&fname)?
        } else if resume {
            resume_from = fs::metadata(&fname)?.len() as usize;
            let message = format!(
                "File exists at: '{}', attempting to resume.",
                fname.to_str().unwrap()
            );
            println!("{}", message);
            let mut existing_file = fs::File::open(&fname)?;
            let mut buffer = vec![0; chunk_size];
            loop {
                let bytes_read = existing_file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            OpenOptions::new().read(true).append(true).open(&fname)?
        } else {
            let message = format!("File exists at: '{}'", fname.to_str().unwrap());
            bail!(message);
        }
    } else {
        // File doesn't exist yet.
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&fname)?
    };
    println!("File will be downloaded to: '{}'.", fname.to_str().unwrap());
    let mut response = if resume_from > 0 {
        println!(
            "Resuming downloading from {}.",
            HumanBytes(resume_from as u64)
        );
        let resp = reqwest::blocking::Client::new()
            .get(&url)
            .header("Range", format!("bytes={}-", resume_from))
            .send()?;
        match resp.status().as_u16() {
            206 => {
                // partial content, resume working
                resp
            }
            416 => {
                // Range not satisfiable, file is likely complete.
                println!("File appears to be complete.");
                bail!("File already complete");
            }
            200 => {
                eprintln!("Server doesn't support resume for this file. Try `--overwrite`");
                bail!("Cannot resume - server sent full file.");
            }
            _ => {
                bail!("Unexpected status: {}", resp.status());
            }
        }
    } else {
        reqwest::blocking::get(&url)?
    };
    let content_length = response.content_length();
    let mut downloaded = resume_from;
    let mut last_update = Instant::now();
    let start_time = Instant::now();
    loop {
        let mut buffer = vec![0; chunk_size];
        let data = response.read(&mut buffer[..])?;
        if data == 0 {
            break;
        }
        downloaded += data;
        if interrupted.load(Ordering::SeqCst) {
            break;
        }
        if last_update.elapsed() >= Duration::from_secs(1) {
            let speed = (downloaded - resume_from) as u64 / start_time.elapsed().as_secs().max(1);
            match content_length {
                Some(len) => bar.set_message(format!(
                    "Downloaded {}/{}. Speed: {}/s. Time Elapsed: {}.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(len),
                    HumanBytes(speed),
                    HumanDuration(start_time.elapsed()),
                )),
                None => bar.set_message(format!(
                    "Downloaded {}. Speed: {} per second. Time Elapsed: {}.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(speed),
                    HumanDuration(start_time.elapsed()),
                )),
            };
            last_update = Instant::now();
        }
        hasher.update(&buffer[..data]);
        dest.write_all(&mut buffer[..data])?;
    }
    dest.sync_all()?;
    if interrupted.load(Ordering::SeqCst) {
        match content_length {
            Some(len) => bar.abandon_with_message(format!(
                "Download interrupted at {}/{}.",
                HumanBytes(downloaded as u64),
                HumanBytes(len),
            )),
            None => bar.abandon_with_message(format!(
                "Download interrupted at {}",
                HumanBytes(downloaded as u64)
            )),
        }
        eprintln!("Download cancelled!");
        bail!("Download cancelled by user.");
    }
    let speed = (downloaded - resume_from) as u64 / start_time.elapsed().as_secs().max(1);
    bar.finish_with_message(format!(
        "Downloaded {} at {}/s in {}.",
        HumanBytes((downloaded - resume_from) as u64),
        HumanBytes(speed),
        HumanDuration(start_time.elapsed())
    ));
    let file_metadata = fs::metadata(&fname)?;
    assert_eq!(file_metadata.len(), downloaded as u64);

    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}

#[allow(unused)]
async fn download_range_async(
    url: String,
    target_dir: &Path,
    start: usize,
    end: usize,
    bar: ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> Result<()> {
    unimplemented!()
}

async fn download_file_async(
    url: String,
    target_dir: &Path,
    chunk_size: usize,
    resume: bool,
    overwrite: bool,
    bar: ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> Result<()> {
    use futures::StreamExt;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{Duration, interval};

    let start_time = Instant::now();

    let fname = url.split("/").last().unwrap_or("tmp.bin");
    let fname = target_dir.join(fname);
    let mut resume_from = 0;

    let mut hasher = Sha256::new();
    let mut dest = if fname.exists() && fname.is_file() {
        if overwrite {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&fname)
                .await?
        } else if resume {
            resume_from = tokio::fs::metadata(&fname).await?.len() as usize;
            let mut existing_file = tokio::fs::File::open(&fname).await?;
            let mut buffer = vec![0; chunk_size];
            loop {
                let bytes_read = existing_file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                };

                hasher.update(&buffer[..bytes_read]);
            }
            OpenOptions::new().append(true).open(&fname).await?
        } else {
            bail!("File exists");
        }
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&fname)
            .await?
    };
    let mut downloaded = resume_from;

    let response = if resume_from > 0 {
        let resp = reqwest::Client::new()
            .get(&url)
            .header("Range", format!("bytes={}-", resume_from))
            .send()
            .await?;
        match resp.status().as_u16() {
            206 => resp,
            416 => bail!("File already complete"),
            200 => {
                eprintln!("Server doesn't support resume. Try --overwrite");
                bail!("Cannot resume.");
            }
            _ => bail!("Unexpected status: {}", resp.status()),
        }
    } else {
        reqwest::get(&url).await?.error_for_status()?
    };

    let mut stream = response.bytes_stream();
    let mut progress_interval = interval(Duration::from_secs(1));
    let mut interrupt_interval = interval(Duration::from_millis(500));
    loop {
        tokio::select! {
            chunk_option = stream.next() => {
                match chunk_option {
                    Some(chunk_result) => {
                    let chunk = chunk_result?;
                    dest.write_all(&chunk).await?;
                    hasher.update(&chunk);
                    downloaded += chunk.len();

                }
                None => break,
            }
            }
            _ = interrupt_interval.tick() => {
                if interrupted.load(Ordering::SeqCst) {
                    let err_message = "Download interrupted.";
                    bar.abandon_with_message(err_message);
                    bail!(err_message);
                }
            }
            _ = progress_interval.tick() => {
                let speed = (downloaded - resume_from) as u64 / start_time.elapsed().as_secs().max(1);
                let message = format!("Downloaded: {}, speed: {}/s. Time Elapsed: {}.", HumanBytes(downloaded as u64), HumanBytes(speed), HumanDuration(start_time.elapsed()));
                bar.set_message(message);
            }
            else => break,
        }
    }
    let speed = (downloaded - resume_from) as u64 / start_time.elapsed().as_secs().max(1);
    bar.finish_with_message(format!(
        "Downloaded: {}, speed: {}/s. Total Time: {}.",
        HumanBytes(downloaded as u64),
        HumanBytes(speed),
        HumanDuration(start_time.elapsed())
    ));
    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let url = cli.url;
    let target_dir = Path::new(&cli.target_directory);
    fs::create_dir_all(target_dir)?;
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message(format!("Attempting to download {}", url));
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .expect("Could not set keyboard interrupt handler.");
    match cli.command {
        Commands::DownloadAsync {
            range_start,
            range_end,
        } => match (range_start, range_end) {
            (None, None) => {
                download_file_async(
                    url,
                    target_dir,
                    cli.chunk_size,
                    cli.resume,
                    cli.overwrite,
                    bar,
                    interrupted,
                )
                .await
            }
            (Some(start), Some(end)) => {
                download_range_async(url, target_dir, start, end, bar, interrupted).await
            }
            _ => {
                bail!("Both --range-start and --range-end are required. You cannot pass only one.",);
            }
        },
        Commands::DownloadBlocking => {
            let target_dir = cli.target_directory.clone();
            tokio::task::spawn_blocking(move || {
                let target_dir = Path::new(&target_dir);
                download_file_blocking(
                    url,
                    target_dir,
                    cli.chunk_size,
                    cli.resume,
                    cli.overwrite,
                    bar,
                    interrupted,
                )
            })
            .await?
        }
    }
}
