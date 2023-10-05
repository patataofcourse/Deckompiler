#![allow(deprecated)]

use clap::Parser;
use std::{fs, path::PathBuf};
use tickflow_parse::Result;

fn main() {
    match run() {
        Ok(_) => (),
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
    /// The tickflow files to compile
    in_: PathBuf,
    /// Location for files to be compiled to
    out: PathBuf,
    #[clap(hide = true, short = 't')]
    /// for le testing
    tickompiler_path: Option<PathBuf>,
}
