use std::path::PathBuf;

use clap::Parser;
use orfail::Result;

#[derive(Debug, Parser)]
struct Args {
    json_file_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    Ok(())
}
