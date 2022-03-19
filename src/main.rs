use deckompiler::btks::BTKS;
use std::{fs::File, io::Result as IOResult};

fn main() -> IOResult<()> {
    let mut f = File::open("test_files/remixTemplate.bin")?;
    let size = f.metadata()?.len();
    let btks = BTKS::from_tickompiler_binary(&mut f, size)?;
    drop(f);
    let mut f = File::create("test_files/out.btk")?;
    btks.to_btks_file(&mut f)
}
