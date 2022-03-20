use crate::common::Tempo;
use bytestream::{ByteOrder, StreamReader, StreamWriter};
use std::io::{Read, Result as IOResult, Seek, SeekFrom, Write};

pub mod constants;
pub mod operations;

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
    pub fn base_offset(&self) -> u64 {
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
    pub fn base_offset(&self) -> u64 {
        self.c00_type.base_offset()
    }

    pub fn from_file<F: Read + Seek>(file: &mut F, c00_type: C00Type) -> IOResult<Self> {
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
            let mut queue = vec![(game.start, 0xFF), (game.assets, 0xFF)];
            let mut bindata = vec![];
            let mut stringdata = vec![];
            while queue.len() != 0 {
                extract_tickflow(&c00_type, file, &mut queue, &mut bindata, &mut stringdata)?;
                queue.remove(0);
            }
        }

        // Step 3 - Read and extract .tempo-s

        // Step 4 - profit

        unimplemented!();
    }
}

/// Equivalent to Tickompiler's firstPass
pub fn extract_tickflow<F: Read + Seek>(
    c00_type: &C00Type,
    file: &mut F,
    queue: &mut Vec<(u32, u32)>,
    bindata: &mut Vec<u8>,
    stringdata: &mut Vec<u8>,
) -> IOResult<()> {
    let mut scene = queue[0].1;
    file.seek(SeekFrom::Start(queue[0].0 as u64 - c00_type.base_offset()))?;
    loop {
        let op_int = u32::read_from(file, ByteOrder::LittleEndian)?;
        let arg_count = (op_int & 0x3C00 >> 10) as u8;
        let mut args = vec![];
        let mut depth = 0;
        for _ in 0..arg_count {
            args.push(u32::read_from(file, ByteOrder::LittleEndian)?);
        }
        if operations::is_scene_op(op_int) {
            scene = *args.get(0).unwrap_or(&scene);
        } else if let Some(c) = operations::is_call_op(op_int) {
            //TODO: mark as pointer for second pass
            //TODO: ok but what func does it refer to??? keep a list somewhere
            queue.push((args[c.args[0] as usize], scene));
        } else if let Some(c) = operations::is_string_op(op_int) {
            //TODO: mark as string pointer for second pass
            //TODO: push position of string pointer
            read_string(
                c00_type,
                file,
                args[c.args[0] as usize].into(),
                c.is_unicode,
            )?;
        }
        //TODO
    }
    Ok(())
}

pub fn read_string<F: Read + Seek>(
    c00_type: &C00Type,
    file: &mut F,
    pos: u64,
    is_unicode: bool,
) -> IOResult<Vec<u8>> {
    let og_pos = file.stream_position()?;
    file.seek(SeekFrom::Start(pos - c00_type.base_offset()))?;
    let mut string_data = vec![];

    if is_unicode {
        loop {
            let chr = u16::read_from(file, ByteOrder::LittleEndian)?;
            string_data.extend(chr.to_le_bytes());
            if chr == 0 {
                break;
            }
        }
    } else {
        loop {
            unimplemented!(); //TODO
        }
    }

    file.seek(SeekFrom::Start(og_pos))?;
    Ok(string_data)
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
