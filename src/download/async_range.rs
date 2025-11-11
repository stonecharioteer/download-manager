#![allow(unused)]
use anyhow::bail;
use indicatif::ProgressBar;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use url::Url;

pub async fn download_range_async(
    url: Url,
    target_dir: &Path,
    start: usize,
    end: usize,
    bar: ProgressBar,
    interrupted: Arc<AtomicBool>,
) -> anyhow::Result<PathBuf> {
    unimplemented!()
}
