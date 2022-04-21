use deckompiler::{
    btks::BTKS,
    c00::{constants::NAME_TEMPO, C00Bin, C00Type},
};
use std::{
    fs::File,
    io::{Result as IOResult, Write},
};

fn main() -> IOResult<()> {
    /*
    let mut f = File::open("test_files/remixTemplate.bin")?;
    let size = f.metadata()?.len();
    let btks = BTKS::from_tickompiler_binary(&mut f, size)?;
    drop(f);
    let mut f = File::create("test_files/out.btk")?;
    btks.to_btks_file(&mut f)
    */

    let mut f = File::open("test_files/C00.bin")?;
    let c00 = C00Bin::from_file(&mut f, C00Type::RHMPatch)?;
    let tfbin = c00.tickflows[0].clone();
    let mut bin = File::create(&format!("test_files/{}", tfbin.name))?;
    tfbin.to_file(&mut bin)?;
    let tempo = c00.tempos[0].clone();
    let mut tmp = File::create(&format!(
        "test_files/{}.tempo",
        NAME_TEMPO[tempo.id as usize - 0x1000000]
    ))?;
    tmp.write(tempo.to_tickompiler_file().as_bytes())?;
    Ok(())
}
