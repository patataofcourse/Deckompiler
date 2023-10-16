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
        /// The path for the output .btk file (defaults to BIN with .btk extension)
        btks: Option<PathBuf>,
        /// Optional tempo files to include in the .btk
        tempo: Vec<PathBuf>,
    },
}

fn main() -> Result<(), i32> {
    match run() {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {}", e);
            Err(1)
        }
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
            let btks_path = match btks_path {
                Some(c) => c,
                None => bin.with_extension("btk"),
            };

            let mut f = File::open(bin)?;
            let size = f.metadata()?.len();
            let (btks, gprac) = BTKS::extract_tickflow(&mut f, size, tempo)?;
            let mut f = File::create(btks_path.clone())?;
            btks.to_btks_file(&mut f)?;
            if let Some(c) = gprac {
                let mut f = File::create(btks_path.with_extension("gprac.btk"))?;
                c.to_btks_file(&mut f)?;
            }
        }
    }
    Ok(())
}
