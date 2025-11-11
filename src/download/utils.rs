use anyhow::Result;
use std::path::{Path, PathBuf};
use url::Url;

pub fn build_download_path(url: &Url, target_dir: &Path) -> PathBuf {
    target_dir.join(
        url.path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("tmp.bin"),
    )
}

pub fn hash_file(path: &Path, chunk_size: usize) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; chunk_size];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize().into())
}
