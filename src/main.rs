#![allow(unused)]

use clap::Parser;
mod cli;
mod download;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    cli.execute().await
}
