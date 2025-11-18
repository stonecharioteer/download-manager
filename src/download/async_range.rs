use crate::download::progress::{ChunkProgressBar, ChunkState};
use crate::download::utils;
use anyhow::bail;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::time::{Duration, interval};
use url::Url;

pub async fn get_content_length(url: &Url) -> anyhow::Result<u64> {
    let response = reqwest::Client::new().get(url.as_str()).send().await?;

    response.content_length().ok_or_else(|| {
        anyhow::anyhow!("Content length not available")
    })
}

pub async fn download_with_workers(
    url: Url,
    target_dir: &Path,
    workers: u8,
    progress: ChunkProgressBar,
    no_cleanup: bool,
) -> anyhow::Result<PathBuf> {
    let content_length = get_content_length(&url).await?;

    let chunk_size = content_length / workers as u64;
    let mut chunks_array: Vec<(usize, usize)> = vec![];

    for i in 0..workers {
        progress.set_chunk_state(i as usize, ChunkState::Pending);
        let start = i as u64 * chunk_size;
        let end = if i == workers - 1 {
            content_length - 1 // last chunk goes to end
        } else {
            (i + 1) as u64 * chunk_size - 1
        };
        chunks_array.push((start as usize, end as usize));
    }

    let mut tasks = Vec::new();
    for (chunk_id, (start, end)) in chunks_array.into_iter().enumerate() {
        let url_clone = url.clone();
        let target_dir = target_dir.to_path_buf();
        let progress_clone = progress.clone();

        let task = tokio::spawn(async move {
            download_range_async(url_clone, &target_dir, start, end, chunk_id, progress_clone).await
        });
        tasks.push(task)
    }

    let results = futures::future::join_all(tasks).await;

    // Collect part paths in the same order as chunks_array
    // (results are in the same order as tasks were spawned)
    let mut part_paths = Vec::new();
    for result in results {
        let path = result??;
        part_paths.push(path);
    }
    // No need to sort - tasks were spawned in order, results maintain that order
    let final_path = merge_parts(&part_paths, target_dir, &url, no_cleanup).await?;
    Ok(final_path)
}

async fn merge_parts(
    part_paths: &[PathBuf],
    target_dir: &Path,
    url: &Url,
    no_cleanup: bool,
) -> anyhow::Result<PathBuf> {
    let final_path = utils::build_download_path(url, target_dir);

    let mut final_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&final_path)
        .await?;

    for part_path in part_paths {
        let mut part_file = tokio::fs::File::open(part_path).await?;
        tokio::io::copy(&mut part_file, &mut final_file).await?;

        if !no_cleanup {
            tokio::fs::remove_file(part_path).await?;
        }
    }

    Ok(final_path)
}

async fn download_range_async(
    url: Url,
    target_dir: &Path,
    start: usize,
    end: usize,
    chunk_id: usize,
    progress: ChunkProgressBar,
) -> anyhow::Result<PathBuf> {
    let _start_time = Instant::now();
    let fname = utils::build_download_path(&url, &target_dir);
    let base_name = fname
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string_lossy();
    let fname = target_dir.join(format!("{base_name}.part.{start}-{end}"));

    let mut dest = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&fname)
        .await?;

    let mut downloaded = 0;

    // Mark this chunk as downloading
    progress.set_chunk_state(chunk_id, ChunkState::Downloading { worker_id: chunk_id });

    let response = reqwest::Client::new()
        .get(url)
        .header("Range", format!("bytes={}-{}", start, end))
        .send()
        .await?;

    let response = match response.status().as_u16() {
        206 => response,
        200 => {
            let message = "Server doesn't support the `range` header, cannot download chunks.";
            eprintln!("{}", message);
            progress.set_chunk_state(chunk_id, ChunkState::Failed);
            bail!(message);
        }
        _ => {
            progress.set_chunk_state(chunk_id, ChunkState::Failed);
            bail!("Unexpected status: {}", response.status())
        }
    };
    let _content_length = response.content_length();

    let mut stream = response.bytes_stream();
    let mut interrupt_interval = interval(Duration::from_millis(500));
    loop {
        tokio::select! {
            chunk_option = stream.next() => {
                match chunk_option {
                    Some(chunk_result) => {
                        let chunk = chunk_result?;
                        dest.write_all(&chunk).await?;
                        downloaded += chunk.len();
                        progress.update_chunk_bytes(chunk_id, downloaded);
                    },
                    None => break,
                }
            }
            _ = interrupt_interval.tick() => {
                if progress.interrupted.load(Ordering::SeqCst) {
                    progress.set_chunk_state(chunk_id, ChunkState::Failed);
                    bail!("Download interrupted.");
                }
            }
        }
    }

    // Mark this chunk as completed
    progress.set_chunk_state(chunk_id, ChunkState::Completed);
    Ok(fname)
}
