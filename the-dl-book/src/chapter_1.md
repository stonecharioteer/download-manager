# Chapter 1 - What is a download anyway?

```admonish abstract title="Chapter Guide"
**What you'll build:** A CLI tool that downloads files over HTTP with concurrent chunks, progress tracking, pause/resume, and SHA256 verification.

**What you'll learn:** Async Rust with tokio, HTTP range requests, atomic progress tracking, state persistence, graceful shutdown.

**End state:** All the work from Tasks 1-8 lives here. By the end of this chapter, you have a working HTTP downloader that handles edge cases and can be paused/resumed.
```

It's 2006, you're downloading a file from the internet. Let's assume that you're
downloading a Linux ISO, you know, legal stuff.

You get the link, which looks like this.

```
https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/x86_64/alpine-standard-3.23.0-x86_64.iso
```

This is a *static download*, a file that's served over HTTP, downloadable using your browser, `curl` or `wget`.
Downloading it is simply sending a HTTP request and receiving a response with some headers and a body, where said body is a stream of bytes representing a file.

```admonish important
The headers aren't really important right now, but you should remember that they exist. These are sadly server-specific, and not all of them are implemented. The right header will tell you how big a file is and you can figure out how much time it could take depending on how much you've already downloaded. If you've ever thought to yourself "Hey, wait a minute, why doesn't this progress bar tell me how much time this download takes?", it's *probably*
because the server didn't tell the client (your browser in this case) how big the file is.
```

Now, how would you write something in Rust that does just that? It should download this file and write to
disk. And since downloads are rather annoying, it should also *verify* said file, using `sha256sum`.

## A Simple Download CLI

```admonish tip
The source code for this part is available in the repo as [v0.1-task1-blocking-mvp](https://github.com/stonecharioteer/download-manager/tree/v0.1-task1-blocking-mvp).
```

### First Steps

First, let's just write the "core" functionality for the above URL.

```rust,linenos
use reqwest::blocking::get;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const URL: &str =
        "https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/x86_64/alpine-standard-3.23.0-x86_64.iso";

    let response = get(URL)?;
    let bytes = response.bytes()?;
    std::fs::write("alpine-standard-3.23.0-x86_64.iso", bytes)?;

    Ok(())
}
```

We're using the `reqwest` crate, with the `reqwuest::blocking::get` method. For now, let's just download this
file using a single-threaded approach, it is fairly inefficient though. We'll update it later, but for now, this is sufficient.

### CLI With `clap`

That's *fairly simple*. Now, let's add a CLI interface so we don't have to hardcode the URL. Here’s the same logic, but driven by a tiny `clap` CLI so the URL (as well as a target directory and overwrite flag) come from the user.

```rust,linenos
use clap::Parser;
use reqwest::blocking::get;

#[derive(Parser)]
struct Cli {
    /// URL to download
    url: String,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let response = get(&cli.url)?;
    let bytes = response.bytes()?;
    let filename = cli.url.split('/').last().unwrap_or("download.bin");
    std::fs::write(&filename, bytes)?;
    println!("Downloaded to {}", destination.display());
    Ok(())
}
```

You'll notice the `cli.url.split('/').last().unwrap_or("download.bin");` section immediately. This only falls back when the url string is empty. The CLI requires a non-empty URL argument, so this almost never triggers. but this prevents against an empty string, like when invoking the binary with `--url ""` perhaps.

### Additional Arguments
Now, let's add some niceties to this command line interface. We could have an argument that sets the directory you'd download said file to, and perhaps one to choose to overwrite the file if it exists.

```rust,linenos
use clap::Parser;
use reqwest::blocking::get;

#[derive(Parser)]
struct Cli {
    /// URL to download
    url: String,

    /// Target directory for the download
    #[arg(long, default_value = ".download")]
    target_dir: String,

    /// overwrite existing file
    #[arg(long)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let response = get(&cli.url)?;
    let bytes = response.bytes()?;
    let filename = cli.url.split('/').last().unwrap_or("download.bin");

    let target_dir = std::path::Path::new(&cli.target_dir);
    std::fs::create_dir_all(target_dir)?;
    let destination = target_dir.join(filename);

    if destination.exists() && !cli.overwrite {
        return Err("file already exists; add --overwrite to replace".into());
    }

    std::fs::write(&destination, bytes)?;
    println!("Downloaded to {}", destination.display());
    Ok(())
}
```

While this works and we *could* stop here, it'd be great to add a progress bar to see what's being downloaded.

### Reporting Progress

The `indicatif` crate is really useful if you want to play with progress bars. To make the download manager a little
more user-friendly, let's add a small spinner that shows off how much of the file is downloaded and what's pending.

```rust,linenos
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::get;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Parser)]
struct Cli {
    /// URL to download
    url: String,

    /// Target directory for the download
    #[arg(long, default_value = ".download")]
    target_dir: String,

    /// Overwrite existing file
    #[arg(long)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let response = get(&cli.url)?;
    let total_bytes = response.content_length().unwrap_or(0);
    let bar = ProgressBar::new(total_bytes);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
            .unwrap(),
    );

    let filename = cli.url.split('/').last().unwrap_or("download.bin");
    let target_dir = Path::new(&cli.target_dir);
    std::fs::create_dir_all(target_dir)?;
    let destination = target_dir.join(filename);

    if destination.exists() && !cli.overwrite {
        let message = format!("File exists at: '{}'", destination.display());
        return Err(message.into());
    }

    let mut file = File::create(&destination)?;
    let mut stream = response;
    let mut buffer = [0_u8; 32 * 1024];

    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        bar.inc(bytes_read as u64);
    }

    bar.finish_with_message("download complete");
    Ok(())
}
```

There are still some improvements we can make here.

### Bringing it All Together

```rust,linenos
use hex;
use indicatif::{HumanBytes, ProgressBar};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use std::time::Instant;
use clap::Parser;

/// Download manager application.
#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// URL to a file to download
    url: String,

    /// Target directory
    #[arg(short, long, default_value = ".download")]
    target_directory: String,

    /// Download chunk size
    #[arg(short, long, default_value_t = 65_536)]
    chunk_size: usize,

    /// Resume if the file already exists and isn't complete
    #[arg(short, long)]
    resume: bool,

    /// Overwrite existing file
    #[arg(short, long)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let target_file = cli.url;
    let target_dir = Path::new(&cli.target_directory);
    fs::create_dir_all(target_dir)?;

    let fname = target_file.split('/').last().unwrap_or("tmp.bin");
    let fname = target_dir.join(fname);
    println!("File to download: '{}'.", fname.to_str().unwrap());
    let mut dest = if fname.exists() && fname.is_file() {
        if cli.overwrite {
            // Overwrite is on, so need to truncate the file.
            let message = format!("File exists at: '{}' overwriting.", fname.to_str().unwrap());
            println!("{}", message);

            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(&fname)?
        } else if cli.resume {
            let message = format!(
                "File exists at: '{}', attempting to resume.",
                fname.to_str().unwrap()
            );
            println!("{}", message);
            OpenOptions::new().read(true).append(true).open(&fname)?
        } else {
            let message = format!("File exists at: '{}'", fname.to_str().unwrap());
            return Err(message.into());
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
    let resume_from = if fname.exists() {
        fs::metadata(&fname)?.len() as usize
    } else {
        0
    };

    let mut response = if resume_from > 0 {
        println!(
            "Resuming downloading from {}.",
            HumanBytes(resume_from as u64)
        );
        let resp = reqwest::blocking::Client::new()
            .get(&target_file)
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
                return Err("File already complete".into());
            }
            200 => {
                eprintln!("Server doesn't support resume for this file. Try `--overwrite`");
                return Err("Cannot resume - server sent full file.".into());
            }
            _ => {
                return Err(format!("Unexpected status: {}", resp.status()).into());
            }
        }
    } else {
        reqwest::blocking::get(&target_file)?
    };

    let mut hasher = Sha256::new();
    let chunk_size = cli.chunk_size;
    let content_length = response.content_length();
    let mut downloaded = resume_from;
    if resume_from > 0 {
        let mut existing_file = fs::File::open(&fname)?;
        let mut buffer = vec![0; chunk_size];
        loop {
            let bytes_read = existing_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    };
    let bar = ProgressBar::new_spinner();
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .expect("Could not set ctrlc handler");

    bar.enable_steady_tick(Duration::from_millis(100));
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
                    "Downloaded {}/{}. Speed: {} per second.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(len),
                    HumanBytes(speed),
                )),
                None => bar.set_message(format!(
                    "Downloaded {}. Speed: {} per second.",
                    HumanBytes(downloaded as u64),
                    HumanBytes(speed)
                )),
            };
            last_update = Instant::now();
        }
        hasher.update(&buffer[..data]);
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
        return Err("Download cancelled by user.".into());
    }
    let speed = downloaded as u64 / start_time.elapsed().as_secs().max(1);
    bar.finish_with_message(format!(
        "Downloaded {} at {} per second.",
        HumanBytes(downloaded as u64),
        HumanBytes(speed)
    ));
    let file_metadata = fs::metadata(&fname)?;
    assert_eq!(file_metadata.len(), downloaded as u64);

    let result = hex::encode(hasher.finalize());
    println!("Sha256sum: {:?}", result);
    Ok(())
}
```

## Downloading in Parallel

Now, let's rethink this problem. I said earlier that the `reqwest::blocking::get` is inefficient and we need to rethink it.
