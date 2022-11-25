use crate::common::{Tempo, TempoVal};
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
    pub fn base_offset(&self) -> u32 {
        match self {
            Self::RHMPatch => 0x0C000000,
            Self::SaltwaterUS | Self::SaltwaterEU | Self::SaltwaterJP | Self::SaltwaterKR => {
                0x060A9008
            }
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
    pub unk: u32,
    pub pos: u32,
}

#[derive(Debug, Clone)]
pub enum Pointer {
    String { offset: u32, points_to: u32 },
    Tickflow { offset: u32, points_to: u32 },
}

impl C00Bin {
    pub fn base_offset(&self) -> u32 {
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
            let assets = u32::read_from(file, ByteOrder::LittleEndian)?;
            if start >= 0x550000 {
                edited_games.push(TickompilerBinary {
                    index: i,
                    start,
                    assets,
                    data: vec![],
                });
            }
            file.seek(SeekFrom::Current(0x28))?;
        }
        file.seek(SeekFrom::Current(0x38))?; // This Shit Should Not Be In Base Dot Bin

        //tempo table
        for _ in 0..0x1E0 {
            let id1 = u32::read_from(file, ByteOrder::LittleEndian)?; // padding
            let id2 = u32::read_from(file, ByteOrder::LittleEndian)?;
            let unk = u32::read_from(file, ByteOrder::LittleEndian)?;
            let pos = u32::read_from(file, ByteOrder::LittleEndian)?;
            if pos >= 0x550000 {
                edited_tempos.push(TempoTable { id1, unk, pos, id2 });
            }
        }

        //gate table
        for i in 0x100..0x110 {
            file.seek(SeekFrom::Current(4))?;
            let start = u32::read_from(file, ByteOrder::LittleEndian)?;
            let assets = u32::read_from(file, ByteOrder::LittleEndian)?;
            if start >= 0x550000 {
                edited_games.push(TickompilerBinary {
                    index: i,
                    start,
                    assets,
                    data: vec![],
                });
            }
            file.seek(SeekFrom::Current(0x18))?;
        }

        // Step 2 - Read and extract tickflow .bin-s
        for game in &mut edited_games {
            let mut func_order = vec![];
            let mut func_positions = vec![];
            let mut queue = vec![(game.start, 0xFF)];
            if game.assets >= 0x550000 {
                queue.push((game.assets, 0xFF));
            }
            let mut bindata = vec![];
            let mut stringdata = vec![];
            let mut pos = 0;
            let mut str_pointers = vec![];
            let mut argann_size = 0;
            while pos < queue.len() {
                func_order.push(queue[pos].0 - c00_type.base_offset());
                func_positions.push(bindata.len() - argann_size);
                str_pointers.extend(extract_tickflow(
                    &c00_type,
                    file,
                    &mut queue,
                    pos,
                    &mut bindata,
                    &mut stringdata,
                    &mut argann_size,
                )?);

                pos += 1;
            }

            let bin_len = bindata.len() - argann_size;

            for pointer in str_pointers {
                if let Pointer::String { offset, points_to } = pointer {
                    let out_bytes = (points_to + bin_len as u32).to_le_bytes();
                    let mut i = 0;
                    for byte in &mut bindata[offset as usize..offset as usize + 4] {
                        *byte = out_bytes[i];
                        i += 1;
                    }
                } else if let Pointer::Tickflow { offset, points_to } = pointer {
                    let func_number = func_order
                        .iter()
                        .position(|&r| r == points_to)
                        .expect("Couldn't find sub");
                    let func_pos = func_positions[func_number];
                    let out_bytes = (func_pos).to_le_bytes();
                    let mut i = 0;
                    for byte in &mut bindata[offset as usize..offset as usize + 4] {
                        *byte = out_bytes[i];
                        i += 1;
                    }
                }
            }

            // End Tickflow, string data only
            0xFFFFFFFEu32.write_to(&mut bindata, ByteOrder::LittleEndian)?;
            (&mut bindata).write(&stringdata)?;

            game.data = bindata;
            game.assets = func_positions[1] as u32;
            game.start = 0;
        }

        // Step 3 - Read and extract .tempo-s
        let mut tempos = vec![];
        for tempo in &edited_tempos {
            // Note: for most (if not all) tempos, one ID is 0xFFFFFFFF
            let mut tempo_vals = vec![];
            file.seek(SeekFrom::Start(
                tempo.pos as u64 - c00_type.base_offset() as u64,
            ))?;
            loop {
                let beats_bytes = u32::read_from(file, ByteOrder::LittleEndian)?;
                let beats = f32::from_bits(beats_bytes);
                let time = u32::read_from(file, ByteOrder::LittleEndian)?;
                let loop_val = u32::read_from(file, ByteOrder::LittleEndian)?;
                tempo_vals.push(TempoVal {
                    beats,
                    time,
                    loop_val,
                });
                //pretty sure this is the observable behavior
                if loop_val & 0x8001 != 0 {
                    break;
                }
            }
            if tempo.id1 != 0xFFFFFFFF {
                tempos.push(Tempo {
                    id: tempo.id1,
                    data: tempo_vals.clone(),
                })
            }
            if tempo.id2 != 0xFFFFFFFF {
                tempos.push(Tempo {
                    id: tempo.id2,
                    data: tempo_vals,
                })
            }
        }

        // Step 4 - profit

        Ok(C00Bin {
            c00_type,
            base_patch: Patch, //TODO
            tickflows: edited_games,
            tempos,
        })
    }
}

/// Equivalent to Tickompiler's firstPass
pub fn extract_tickflow<F: Read + Seek>(
    c00_type: &C00Type,
    file: &mut F,
    queue: &mut Vec<(u32, u32)>,
    pos: usize,
    bindata: &mut Vec<u8>,
    stringdata: &mut Vec<u8>,
    argann_size: &mut usize,
) -> IOResult<Vec<Pointer>> {
    let mut scene = queue[pos].1;
    file.seek(SeekFrom::Start(
        queue[pos].0 as u64 - c00_type.base_offset() as u64,
    ))?;
    let mut done = false;
    let mut pointers = vec![];
    let mut depth = 0;
    while !done {
        let op_int = u32::read_from(file, ByteOrder::LittleEndian)?;
        let arg_count = ((op_int & 0x3C00) >> 10) as u8;
        let mut args = vec![];
        for _ in 0..arg_count {
            args.push(u32::read_from(file, ByteOrder::LittleEndian)?);
        }
        if operations::is_scene_op(op_int) {
            scene = args[0];
        } else if let Some(c) = operations::is_call_op(op_int) {
            let pointer_pos = args[c.args[0] as usize];
            if pointer_pos < c00_type.base_offset() {
                println!("called game tickflow at 0x{:08X}", pointer_pos);
            } else {
                let mut is_in_queue = false;
                for (position, _) in &*queue {
                    if *position == pointer_pos {
                        is_in_queue = true;
                        break;
                    }
                }
                if !is_in_queue {
                    queue.push((pointer_pos, scene));
                }
                args[c.args[0] as usize] = pointer_pos - c00_type.base_offset();

                // Tickompiler argument annotation
                //  0xFFFFFFFF - Start section
                //  0x00000001 - One argument
                //  0x00000X00 - Pointer argument at position X = c.args[0]
                (0xFFFFFFFFu32).write_to(bindata, ByteOrder::LittleEndian)?;
                1u32.write_to(bindata, ByteOrder::LittleEndian)?;
                ((c.args[0] as u32) << 8).write_to(bindata, ByteOrder::LittleEndian)?;
                *argann_size += 3 * 4;

                pointers.push(Pointer::Tickflow {
                    offset: bindata.len() as u32 + (4 * (c.args[0] + 1)) as u32,
                    points_to: pointer_pos - c00_type.base_offset(),
                });
            }
        } else if let Some(c) = operations::is_string_op(op_int, scene) {
            // Tickompiler argument annotation
            //  0xFFFFFFFF - Start section
            //  0x0000000X - X arguments
            //  For each argument:
            //    0x00000X0Y - Pointer argument at position X = arg of type Y = 1 if unicode, 2 if ASCII
            (0xFFFFFFFFu32).write_to(bindata, ByteOrder::LittleEndian)?;
            (c.args.len() as u32).write_to(bindata, ByteOrder::LittleEndian)?;
            for arg in &c.args {
                let p_type = if c.is_unicode { 1 } else { 2 };
                (p_type + ((*arg as u32) << 8)).write_to(bindata, ByteOrder::LittleEndian)?;
            }
            *argann_size += (2 + c.args.len()) * 4;

            for arg in &c.args {
                pointers.push(Pointer::String {
                    offset: bindata.len() as u32 + (4 * (*arg + 1)) as u32,
                    points_to: stringdata.len() as u32,
                });

                stringdata.extend(read_string(
                    c00_type,
                    file,
                    args[*arg as usize].into(),
                    c.is_unicode,
                )?);
            }
        } else if let Some(_) = operations::is_depth_op(op_int) {
            #[allow(unused_assignments)]
            {
                depth += 1;
            }
        } else if let Some(_) = operations::is_undepth_op(op_int) {
            #[allow(unused_assignments)]
            {
                if depth > 0 {
                    depth -= 1;
                }
            }
        } else if let Some(_) = operations::is_return_op(op_int) {
            if depth <= 0 {
                done = true;
            }
        }
        op_int.write_to(bindata, ByteOrder::LittleEndian)?;
        for arg in args {
            arg.write_to(bindata, ByteOrder::LittleEndian)?;
        }
    }
    Ok(pointers)
}

pub fn read_string<F: Read + Seek>(
    c00_type: &C00Type,
    file: &mut F,
    pos: u64,
    is_unicode: bool,
) -> IOResult<Vec<u8>> {
    let og_pos = file.stream_position()?;
    if pos < c00_type.base_offset() as u64 {
        return Ok(vec![0, 0]);
    }
    file.seek(SeekFrom::Start(pos - c00_type.base_offset() as u64))?;
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
            let chr = u8::read_from(file, ByteOrder::LittleEndian)?;
            string_data.push(chr);
            if chr == 0 {
                break;
            }
        }
    }

    //padding
    for _ in 0..(4 - string_data.len() % 4) {
        string_data.push(0);
    }

    file.seek(SeekFrom::Start(og_pos))?;
    Ok(string_data)
}

impl TickompilerBinary {
    pub fn to_file<F: Write + Seek>(&self, file: &mut F) -> IOResult<()> {
        self.index.write_to(file, ByteOrder::LittleEndian)?;
        self.start.write_to(file, ByteOrder::LittleEndian)?;
        self.assets.write_to(file, ByteOrder::LittleEndian)?;
        file.write(&self.data)?;
        if file.stream_position()? % 4 != 0 {
            for _ in 0..8 - (file.stream_position()? % 4) {
                0u8.write_to(file, ByteOrder::LittleEndian)?;
            }
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        if self.index >= 0x100 {
            constants::NAME_TICKFLOW_ENDLESS[self.index as usize - 0x100]
        } else {
            constants::NAME_TICKFLOW[self.index as usize]
        }
    }
}

impl Tempo {
    pub fn to_tickompiler_file(&self) -> String {
        let mut out = format!("{:X}\n", self.id);
        for val in &self.data {
            let seconds = val.time as f64 / 32000.0;
            let bpm = 60.0 * val.beats as f64 / seconds;
            out += &format!("{:#.3} {:#.3} {}\n", bpm, val.beats as f64, val.loop_val);
        }
        out
    }

    pub fn name(&self) -> &str {
        constants::NAME_TEMPO[self.id as usize - 0x1000000]
    }
}
