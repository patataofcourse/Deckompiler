use crate::common::Tempo;
use std::io::{Read, Result as IOResult, Seek};

pub mod constants;
pub mod string;

#[derive(Debug)]
pub struct C00Bin {
    pub c00_type: C00Type,
    pub base_patch: Patch, //TODO: IPS??? custom format??? to be decided
    pub tickflows: Vec<TickompilerBinary>,
    pub tempos: Vec<Tempo>,
}

//stub
#[derive(Debug, Clone)]
pub struct Patch;

#[derive(Debug, Clone)]
pub enum C00Type {
    RHMPatch,
    SaltwaterUS,
    SaltwaterEU,
    SaltwaterJP,
    SaltwaterKR,
}

impl C00Type {
    pub fn base_offset(&self) -> u32 {
        match self {
            Self::RHMPatch => 0xC0000000,
            _ => panic!("Saltwater base offset not implemented"), //TODO
        }
    }
}

#[derive(Debug, Clone)]
pub struct TickompilerBinary {
    pub index: u32,
    pub start: u32,
    pub assets: u32,
    pub data: Vec<u8>,
}

impl C00Bin {
    pub fn base_offset(&self) -> u32 {
        self.c00_type.base_offset()
    }

    pub fn from_file<F: Read + Seek>(file: &mut F) -> IOResult<Self> {
        unimplemented!();
        /*
        # Step 1 - Go through the base.bin tables and try to find the positions
        #    (if they're greater than 0x550000, then it's modded)
        games = []
        tempos = []
        # Game table
        for index in range(0x68):
            c00.read(4)
            start = int.from_bytes(c00.read(4), "little")
            assets = int.from_bytes(c00.read(4), "little")
            if start >= base:
                games.append((index, start, assets))
            c00.read(0x28)
        c00.read(0x68)
        # Tempo table
        for _ in range(0x1DD):
            id1 = int.from_bytes(c00.read(4), "little")
            id2 = int.from_bytes(c00.read(4), "little")
            pos = int.from_bytes(c00.read(4), "little")
            padding = int.from_bytes(c00.read(4), "little")
            if pos >= base:
                tempos.append((id1, id2, pos, padding))
        # Gate table
        for index in range(0x10):
            c00.read(4)
            start = int.from_bytes(c00.read(4), "little")
            assets = int.from_bytes(c00.read(4), "little")
            if start >= base:
                games.append((0x100+index, start, assets))
            c00.read(0x18)

        c00.seek(0)
        c00 = c00.read()
        try:
            os.makedirs(outdir)
        except FileExistsError:
            pass

        # Step 2 - Read and extract tickflow .bin-s
        for game in games:
            is_gate = game[0] >= 0x100
            if is_gate: game[0] -= 0x100
            name = names["tickflowEndless" if is_gate else "tickflow"][game[0]] + ".bin"
        #now we gotta copy a bunch of shit from tickompiler

        # Step 3 - Read and extract .tempo-s

        # Step 4 - profit
        */
    }
}

impl TickompilerBinary {}

impl Tempo {
    pub fn to_tickompiler_file(&self) -> String {
        let mut out = format!("{}\n", self.id);
        for val in &self.data {
            let seconds = val.time as f64 * 1000.0;
            let bpm = val.beats as f64 / seconds * 60.0;
            out += &format!("{:#.3} {:#.3} {}", bpm, seconds, val.loop_val);
        }
        out
    }
}
