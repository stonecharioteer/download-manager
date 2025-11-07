use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use hex;
use sha2::{Digest, Sha256};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// URL to a file to download
    url: String,

    /// Target directory
    #[arg(short, long, default_value = ".download")]
    target_directory: String,

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

    let content_length = response.content_length();

    match content_length {
        Some(len) => println!("Downloading {} bytes", len),
        None => println!("Downloading size unknown"),
    };

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
    loop {
        let mut buffer = vec![0; chunk_size];
        let data = response.read(&mut buffer[..])?;
        if data == 0 {
            break;
        }
        hasher.update(&buffer[..data]);
        dest.write_all(&mut buffer[..data])?;
    }
    dest.sync_all()?;

    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}
