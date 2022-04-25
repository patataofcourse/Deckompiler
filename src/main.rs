use clap::{Parser, Subcommand};
use deckompiler::{
    btks::BTKS,
    c00::{C00Bin, C00Type},
};
use std::{
    fs::{self, File},
    io::{Result as IOResult, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert a Tickompiler .bin file to a Spicerack .btk
    Btks {
        /// The path of the input .bin file
        bin: PathBuf,
        /// The path for the output .btk file
        btks: PathBuf,
        /// [to be implemented] Optional tempo files to include in the .btk
        tempo: Vec<PathBuf>,
    },
    /// Extracts all contents of a C00.bin file into a folder
    C00 {
        /// The C00.bin file to extract
        c00: PathBuf,
        /// Location for files to be extracted
        out: PathBuf,
    },
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> IOResult<()> {
    let cli = Cli::parse();
    match cli.commands {
        Commands::Btks {
            bin,
            btks: btks_path,
            tempo,
        } => {
            let mut f = File::open(bin)?;
            let size = f.metadata()?.len();
            let btks = BTKS::from_tickompiler_binary(&mut f, size)?;
            //TODO: tempos
            let mut f = File::create(btks_path)?;
            btks.to_btks_file(&mut f)?;
        }
        Commands::C00 { c00, out } => {
            let mut f = File::open(c00)?;
            let c00 = C00Bin::from_file(&mut f, C00Type::RHMPatch)?;
            fs::create_dir_all(&out)?;
            for tfbin in c00.tickflows {
                let mut out = out.clone();
                out.push(PathBuf::from(format!("{}.bin", tfbin.name())));
                let mut bin = File::create(out)?;
                tfbin.to_file(&mut bin)?;
            }
            for tempo in c00.tempos {
                let mut out = out.clone();
                out.push(PathBuf::from(format!("{}.tempo", tempo.name())));
                let mut tfile = File::create(out)?;
                tfile.write(tempo.to_tickompiler_file().as_bytes())?;
            }
        }
    }
    Ok(())
}
