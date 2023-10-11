#![allow(deprecated)]

use clap::Parser;
use std::{fs, path::PathBuf};
use tickflow_parse::{Error, Result};

fn main() {
    match run() {
        Ok(_) => (),
        Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::Other => {
            println!("deckompiler error: {}", e)
        }
        Err(e) => println!("{}", e),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.out.parent().map(fs::create_dir_all).unwrap_or(Ok(()))?;
    deckompiler::compiler::compile_file(
        cli.in_,
        cli.out,
        deckompiler::compiler::CompiledFileType::Tickompiler,
    )?;
    Ok(())
}

#[derive(Parser)]
#[clap(
    name = "Not Tickompiler",
    version,
    author,
    arg_required_else_help = true
)]
/// me when i        when i compile tickflow
struct Cli {
    /// The tickflow file to compile
    in_: PathBuf,
    /// Location for the file to be compiled to
    out: PathBuf,
}
