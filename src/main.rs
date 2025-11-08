use hex;
use indicatif::{HumanBytes, ProgressBar};
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

use clap::{Parser, Subcommand};

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
    DownloadAsync,
}

fn download_file_blocking(
    url: String,
    target_dir: &Path,
    chunk_size: usize,
    resume: bool,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let fname = url.split('/').last().unwrap_or("tmp.bin");
    let fname = target_dir.join(fname);
    println!("File to download: '{}'.", fname.to_str().unwrap());
    let mut dest = if fname.exists() && fname.is_file() {
        if overwrite {
            // Overwrite is on, so need to truncate the file.
            let message = format!("File exists at: '{}' overwriting.", fname.to_str().unwrap());
            println!("{}", message);

            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(&fname)?
        } else if resume {
            let message = format!(
                "File exists at: '{}', attempting to resume.",
                fname.to_str().unwrap()
            );
            println!("{}", message);
            OpenOptions::new().read(true).append(true).open(&fname)?
        } else {
            let message = format!("File exists at: '{}'", fname.to_str().unwrap());
            return Err(message.into());
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
    let resume_from = if fname.exists() {
        fs::metadata(&fname)?.len() as usize
    } else {
        0
    };

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
                return Err("File already complete".into());
            }
            200 => {
                eprintln!("Server doesn't support resume for this file. Try `--overwrite`");
                return Err("Cannot resume - server sent full file.".into());
            }
            _ => {
                return Err(format!("Unexpected status: {}", resp.status()).into());
            }
        }
    } else {
        reqwest::blocking::get(&url)?
    };

    let mut hasher = Sha256::new();
    let content_length = response.content_length();
    let mut downloaded = resume_from;
    if resume_from > 0 {
        let mut existing_file = fs::File::open(&fname)?;
        let mut buffer = vec![0; chunk_size];
        loop {
            let bytes_read = existing_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    };
    let bar = ProgressBar::new_spinner();
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .expect("Could not set ctrlc handler");

    bar.enable_steady_tick(Duration::from_millis(100));
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
                    "Downloaded {}/{}. Speed: {} per second.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(len),
                    HumanBytes(speed),
                )),
                None => bar.set_message(format!(
                    "Downloaded {}. Speed: {} per second.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(speed)
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
        return Err("Download cancelled by user.".into());
    }
    let speed = (downloaded - resume_from) as u64 / start_time.elapsed().as_secs().max(1);
    bar.finish_with_message(format!(
        "Downloaded {} at {} per second in {} s.",
        HumanBytes((downloaded - resume_from) as u64),
        HumanBytes(speed),
        start_time.elapsed().as_secs()
    ));
    let file_metadata = fs::metadata(&fname)?;
    assert_eq!(file_metadata.len(), downloaded as u64);

    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let url = cli.url;
    let target_dir = Path::new(&cli.target_directory);
    fs::create_dir_all(target_dir)?;

    match cli.command {
        Commands::DownloadAsync => {
            unimplemented!("Async downloads are not yet implemented.")
        }
        Commands::DownloadBlocking => {
            download_file_blocking(url, target_dir, cli.chunk_size, cli.resume, cli.overwrite)
        }
    }
}
