#![allow(deprecated)]

use clap::Parser;
use deckompiler::c00::{C00Bin, C00Type};
use std::{
    fs::{self, File},
    io::{Result as IOResult, Write},
    path::PathBuf,
};

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> IOResult<()> {
    let cli = Cli::parse();
    let mut f = File::open(cli.c00)?;
    let c00 = C00Bin::from_file(&mut f, C00Type::RHMPatch, cli.old)?;
    fs::create_dir_all(&cli.out)?;
    for tfbin in c00.tickflows {
        let mut out = cli.out.clone();
        out.push(PathBuf::from(format!("{}.bin", tfbin.name())));
        let mut bin = File::create(out)?;
        tfbin.to_file(&mut bin)?;
    }
    for tempo in c00.tempos {
        let mut out = cli.out.clone();
        out.push(PathBuf::from(format!("{}.tempo", tempo.name())));
        let mut tfile = File::create(out)?;
        tfile.write(tempo.to_tickompiler_file().as_bytes())?;
    }
    Ok(())
}

#[derive(Parser)]
#[clap(name = "deckompiler-c00", version, author)]
/// Extracts all contents of a C00.bin file into a folder
struct Cli {
    /// The C00.bin file to extract
    c00: PathBuf,
    /// Location for files to be extracted
    out: PathBuf,
    #[clap(
        short = 'o',
        long = "old-c00",
        help = "Enable this if you C00.bin predates the Aug 2017 gate patch"
    )]
    old: bool,
}
