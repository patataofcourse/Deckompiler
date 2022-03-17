use deckompiler::btks::BTKS;
use std::{fs::File, io::Result as IOResult};

fn main() -> IOResult<()>{
    let mut f = File::open("test_files/in.bin")?;
    let size = f.metadata()?.len();
    BTKS::from_tickompiler_binary(&mut f, size)?;
    Ok(())
}
