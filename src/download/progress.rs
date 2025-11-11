use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize},
};

#[derive(Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: Arc<AtomicUsize>,
    pub total_bytes: Arc<AtomicU64>,
    pub interrupted: Arc<AtomicBool>,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            bytes_downloaded: Arc::new(AtomicUsize::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }
}
