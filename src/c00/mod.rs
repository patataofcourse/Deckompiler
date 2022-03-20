use crate::common::Tempo;
use bytestream::{ByteOrder, StreamReader, StreamWriter};
use std::io::{Read, Result as IOResult, Seek, SeekFrom, Write};

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

#[derive(Debug, Clone)]
pub struct TempoTable {
    pub id1: u32,
    pub id2: u32,
    pub pos: u32,
}

impl C00Bin {
    pub fn base_offset(&self) -> u32 {
        self.c00_type.base_offset()
    }

    pub fn from_file<F: Read + Seek>(file: &mut F) -> IOResult<Self> {
        let mut edited_games = vec![];
        let mut edited_tempos = vec![];

        //TODO: detect base.bin patches

        // Step 1 - Go through the base.bin tables and try to find the positions
        //    (if they're greater than 0x550000, then it's modded)

        //game table
        for i in 0..0x68 {
            file.seek(SeekFrom::Current(4))?;
            let start = u32::read_from(file, ByteOrder::LittleEndian)?;
            if start >= 0x550000 {
                edited_games.push(TickompilerBinary {
                    index: i,
                    start,
                    assets: u32::read_from(file, ByteOrder::LittleEndian)?,
                    data: vec![],
                });
            }
            file.seek(SeekFrom::Current(0x28))?;
        }
        file.seek(SeekFrom::Current(0x68))?; // This Shit Should Not Be In Base Dot Bin

        //tempo table
        for _ in 0..0x1DD {
            let id1 = u32::read_from(file, ByteOrder::LittleEndian)?;
            let id2 = u32::read_from(file, ByteOrder::LittleEndian)?;
            let pos = u32::read_from(file, ByteOrder::LittleEndian)?;
            u32::read_from(file, ByteOrder::LittleEndian)?; // padding
            if pos >= 0x550000 {
                edited_tempos.push(TempoTable { id1, id2, pos });
            }
        }

        //gate table
        for i in 0x100..0x110 {
            file.seek(SeekFrom::Current(4))?;
            let start = u32::read_from(file, ByteOrder::LittleEndian)?;
            if start >= 0x550000 {
                edited_games.push(TickompilerBinary {
                    index: i,
                    start,
                    assets: u32::read_from(file, ByteOrder::LittleEndian)?,
                    data: vec![],
                });
            }
            file.seek(SeekFrom::Current(0x18))?;
        }

        // Step 2 - Read and extract tickflow .bin-s
        for game in edited_games {
            let is_gate = game.index >= 0x100;
            let name = if is_gate {
                constants::NAME_TICKFLOW_ENDLESS[game.index as usize - 0x100]
            } else {
                constants::NAME_TICKFLOW[game.index as usize]
            };
            //now we gotta copy a bunch of shit from tickompiler
        }

        // Step 3 - Read and extract .tempo-s

        // Step 4 - profit

        unimplemented!();
    }
}

impl TickompilerBinary {
    pub fn to_file<F: Write>(&self, file: &mut F) -> IOResult<()> {
        self.index.write_to(file, ByteOrder::LittleEndian)?;
        self.start.write_to(file, ByteOrder::LittleEndian)?;
        self.assets.write_to(file, ByteOrder::LittleEndian)?;
        file.write(&self.data)?;
        Ok(())
    }
}

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
