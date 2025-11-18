use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

use colored::Colorize;
use std::time::Instant;

// Trait to homogenize the progress tracking, so we are not dependent on indicatif.
pub trait ProgressTracker: Send + Sync + Clone {
    fn interrupted(&self) -> Arc<AtomicBool>;
    fn update_progress(&self, bytes: usize);
    fn render(&self);
    fn finish(&self, msg: &str);
    fn abandon(&self, msg: &str);
}

#[derive(Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: Arc<AtomicUsize>,
    pub total_bytes: Arc<AtomicU64>,
    pub interrupted: Arc<AtomicBool>,
}

impl DownloadProgress {
    pub fn new(interrupted: Arc<AtomicBool>) -> Self {
        Self {
            bytes_downloaded: Arc::new(AtomicUsize::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            interrupted,
        }
    }
}

#[derive(Clone)]
pub enum ChunkState {
    Pending,
    Downloading { worker_id: usize },
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct ChunkProgressBar {
    bar: indicatif::ProgressBar,
    chunks: Arc<Mutex<Vec<ChunkState>>>,
    bytes_per_chunk: Vec<Arc<AtomicUsize>>,
    total_bytes: u64,
    start_time: Instant,
    pub interrupted: Arc<AtomicBool>,
}

impl ChunkProgressBar {
    pub fn new(num_chunks: usize, total_bytes: u64, interrupted: Arc<AtomicBool>) -> Self {
        let bar = indicatif::ProgressBar::new_spinner();
        bar.enable_steady_tick(std::time::Duration::from_millis(100));
        let chunks = vec![ChunkState::Pending; num_chunks];
        let bytes_per_chunk = (0..num_chunks)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();
        Self {
            bar,
            chunks: Arc::new(Mutex::new(chunks)),
            bytes_per_chunk,
            total_bytes,
            start_time: Instant::now(),
            interrupted,
        }
    }

    pub fn update_chunk_bytes(&self, chunk_id: usize, bytes: usize) {
        if chunk_id < self.bytes_per_chunk.len() {
            self.bytes_per_chunk[chunk_id].store(bytes, Ordering::Relaxed);
        }
    }

    pub fn set_chunk_state(&self, chunk_id: usize, state: ChunkState) {
        if let Ok(mut chunks) = self.chunks.lock() {
            if chunk_id < chunks.len() {
                chunks[chunk_id] = state;
            }
        }
    }

    pub fn get_total_downloaded(&self) -> usize {
        self.bytes_per_chunk
            .iter()
            .map(|bytes| bytes.load(Ordering::Relaxed))
            .sum()
    }
    fn render_chunks(&self) -> String {
        const PROGRESS_CHAR: &str = "█";
        const WIP_CHAR: &str = "░";
        if let Ok(chunks) = self.chunks.lock() {
            let mut output = String::from("[");
            for chunk in chunks.iter() {
                let symbol = match chunk {
                    ChunkState::Completed => PROGRESS_CHAR.green(),
                    ChunkState::Downloading { worker_id } => {
                        // Use different colors for different workers
                        // FIXME: I'm not happy with this implementation, how would
                        // this be useful?
                        match worker_id % 3 {
                            0 => PROGRESS_CHAR.yellow(),
                            1 => PROGRESS_CHAR.cyan(),
                            _ => PROGRESS_CHAR.magenta(),
                        }
                    }
                    // Black? What about a light mode?
                    ChunkState::Pending => WIP_CHAR.bright_black(),
                    ChunkState::Failed => PROGRESS_CHAR.red(),
                };
                output.push_str(&symbol.to_string());
            }

            output.push(']');
            output
        } else {
            String::from("[?]")
        }
    }
}

impl ProgressTracker for ChunkProgressBar {
    fn update_progress(&self, bytes: usize) {
        todo!()
    }
    fn interrupted(&self) -> Arc<AtomicBool> {
        todo!()
    }

    fn render(&self) {
        let total_downloaded = self.get_total_downloaded();
        let elapsed = self.start_time.elapsed().as_secs().max(1);
        let speed = total_downloaded as u64 / elapsed;

        let chunks_viz = self.render_chunks();

        // Build the message
        let message = format!(
            "{} Downloaded: {} / {} @ {}/s",
            chunks_viz,
            indicatif::HumanBytes(total_downloaded as u64),
            indicatif::HumanBytes(self.total_bytes),
            indicatif::HumanBytes(speed),
        );

        self.bar.set_message(message);
    }

    fn finish(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    fn abandon(&self, msg: &str) {
        self.bar.abandon_with_message(msg.to_string());
    }
}
