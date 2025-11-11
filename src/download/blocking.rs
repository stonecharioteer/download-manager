use anyhow::bail;
use indicatif::{HumanBytes, HumanDuration, ProgressBar};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use url::Url;

use crate::download::utils;

pub fn download_file_blocking(
    url: Url,
    target_dir: &PathBuf,
    chunk_size: usize,
    resume: bool,
    overwrite: bool,
    bar: ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> anyhow::Result<PathBuf> {
    let fname = utils::build_download_path(&url, target_dir);
    println!("File to download: '{}'.", fname.to_str().unwrap());
    let mut resume_from = 0;
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
            .get(url)
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
        reqwest::blocking::get(url)?
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

    Ok(fname)
}
