use anyhow::bail;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use url::Url;

use crate::download::progress::DownloadProgress;
use crate::download::utils;

pub fn download_file_blocking(
    url: Url,
    target_dir: &PathBuf,
    chunk_size: usize,
    resume: bool,
    overwrite: bool,
    progress: DownloadProgress,
) -> anyhow::Result<PathBuf> {
    let fname = utils::build_download_path(&url, target_dir);
    let mut resume_from = 0;
    let mut dest = if fname.exists() && fname.is_file() {
        if overwrite {
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(&fname)?
        } else if resume {
            resume_from = fs::metadata(&fname)?.len() as usize;
            OpenOptions::new().read(true).append(true).open(&fname)?
        } else {
            bail!("File exists at '{}'", fname.display());
        }
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&fname)?
    };
    let mut response = if resume_from > 0 {
        let resp = reqwest::blocking::Client::new()
            .get(url)
            .header("Range", format!("bytes={}-", resume_from))
            .send()?;
        match resp.status().as_u16() {
            206 => resp,
            416 => bail!("File already complete"),
            200 => {
                eprintln!("Server doesn't support resume. Try --overwrite");
                bail!("Cannot resume - server sent full file");
            }
            _ => bail!("Unexpected status: {}", resp.status()),
        }
    } else {
        reqwest::blocking::get(url)?
    };
    let content_length = response.content_length();
    progress
        .total_bytes
        .store(content_length.unwrap_or(0), Ordering::Relaxed);
    let mut downloaded = resume_from;
    loop {
        let mut buffer = vec![0; chunk_size];
        let data = response.read(&mut buffer[..])?;
        if data == 0 {
            break;
        }
        downloaded += data;
        progress
            .bytes_downloaded
            .store(downloaded, Ordering::Relaxed);
        if progress.interrupted.load(Ordering::SeqCst) {
            break;
        }
        dest.write_all(&mut buffer[..data])?;
    }
    dest.sync_all()?;

    if progress.interrupted.load(Ordering::SeqCst) {
        bail!("Download cancelled by user");
    }

    Ok(fname)
}
