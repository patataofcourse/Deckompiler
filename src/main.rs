use clap::{Parser, Subcommand};
use deckompiler::btks::BTKS;
use std::{fs::File, io::Result as IOResult, path::PathBuf};

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
    }
    Ok(())
}
