use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = Path::new(".download");
    fs::create_dir_all(target_dir)?;
    let target_file = "https://dl-cdn.alpinelinux.org/alpine/v3.22/releases/x86_64/alpine-standard-3.22.2-x86_64.iso";

    let response = reqwest::blocking::get(target_file)?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");
        println!("File to download: '{}'", fname);
        let fname = target_dir.join(fname);
        println!("File will be located under: '{:?}", fname);
        File::create(fname)?
    };
    let content = response.bytes()?;
    dest.write_all(&content)?;
    Ok(())
}
