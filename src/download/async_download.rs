use anyhow::bail;
use indicatif::{HumanBytes, HumanDuration};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::time::Instant;
use url::Url;

use crate::download::utils;

pub async fn download_file_async(
    url: Url,
    target_dir: &PathBuf,
    resume: bool,
    overwrite: bool,
    bar: indicatif::ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> anyhow::Result<PathBuf> {
    use futures::StreamExt;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{Duration, interval};

    let start_time = Instant::now();

    let fname = utils::build_download_path(&url, &target_dir);
    let mut resume_from = 0;

    let mut dest = if fname.exists() && fname.is_file() {
        if overwrite {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&fname)
                .await?
        } else if resume {
            resume_from = tokio::fs::metadata(&fname).await?.len() as usize;
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
            .get(url)
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
        reqwest::get(url).await?.error_for_status()?
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
    Ok(fname)
}
