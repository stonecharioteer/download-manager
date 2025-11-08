use hex;
use indicatif::{HumanBytes, ProgressBar};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use std::time::Instant;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// URL to a file to download
    url: String,

    /// Target directory
    #[arg(short, long, default_value = ".download")]
    target_directory: String,

    /// Download chunk size
    #[arg(short, long, default_value_t = 65_536)]
    chunk_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let target_file = cli.url;
    let target_dir = Path::new(&cli.target_directory);
    fs::create_dir_all(target_dir)?;

    let mut response = reqwest::blocking::get(target_file)?;

    if !response.status().is_success() {
        return Err("Error getting the file.".into());
    }

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    println!("File to download: '{}'.", fname);
    let fname = target_dir.join(fname);
    println!("File will be located under: '{}'.", fname.to_str().unwrap());
    let mut dest = File::create(&fname)?;
    let mut hasher = Sha256::new();
    let chunk_size = cli.chunk_size;
    let content_length = response.content_length();
    let mut downloaded = 0;
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
            let speed = downloaded as u64 / start_time.elapsed().as_secs();
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
    let speed = downloaded as u64 / start_time.elapsed().as_secs();
    bar.finish_with_message(format!(
        "Downloaded {} at {} per second.",
        HumanBytes(downloaded as u64),
        HumanBytes(speed)
    ));
    let file_metadata = fs::metadata(&fname)?;
    assert_eq!(file_metadata.len(), downloaded as u64);

    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}
