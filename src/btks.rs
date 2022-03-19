use crate::common::Tempo;
use bytestream::{ByteOrder, StreamReader, StreamWriter};
use std::io::{self, Read, Seek, SeekFrom, Write};

#[derive(Debug, Clone)]
pub struct BTKS {
    flow: FlowSection,
    ptro: Option<Vec<Pointer>>,
    tmpo: Option<Vec<Tempo>>,
    strd: Option<Vec<u8>>,
}

impl BTKS {
    const REVISION: u32 = 0;
    const HEADER_SIZE: u32 = 0x10;
    const FLOW_HEADER: u32 = 0xC;
    const PTRO_HEADER: u32 = 0xC;
    //const TMPO_HEADER: u32 = 0x8; //TODO
    const STRD_HEADER: u32 = 0x8;
}

#[derive(Debug, Clone)]
pub struct FlowSection {
    start_offset: u32,
    tickflow_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Pointer {
    offset: u32,
    ptype: PointerType,
}

impl Pointer {
    pub fn to_bin(&self) -> [u8; 5] {
        let mut out = [0; 5];
        for i in 0..4 {
            out[i] = self.offset.to_le_bytes()[i];
        }
        out[4] = self.ptype.clone() as u8;
        out
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PointerType {
    String,
    Tickflow,
}

impl BTKS {
    pub fn from_tickompiler_binary<F: Read + Seek>(f: &mut F, file_size: u64) -> io::Result<Self> {
        //not needed- but nice to print for info purposes
        let index = u32::read_from(f, ByteOrder::LittleEndian)?;
        println!("Index of file: {:#X}", index);
        let start = u32::read_from(f, ByteOrder::LittleEndian)?;
        u32::read_from(f, ByteOrder::LittleEndian)?; //Ignore assets sub
        let mut tickflow = vec![];
        // .bin tickflow loop or whatever
        // copied from tickompiler, modified to export to btks
        let mut pointers = vec![];
        loop {
            let mut cmd = u32::read_from(f, ByteOrder::LittleEndian)?;
            if cmd == 0xFFFFFFFE {
                // 0xFFFFFFFE (-2) indicates start of string data
                break;
            }
            let mut str_args = vec![]; //strings and tickflow pointers have to be stored separately
            let mut ptr_args = vec![]; //because they're managed differently in btks
            if cmd == 0xFFFFFFFF {
                // 0xFFFFFFFF (-1) indicates an 'args' section
                let amount = u32::read_from(f, ByteOrder::LittleEndian)?;
                for _ in 0..amount {
                    let ann = u32::read_from(f, ByteOrder::LittleEndian)?;
                    let anncode = ann & 0xFF;
                    let ann_arg = (ann & 0xFFFFFF) >> 8;
                    if anncode == 0 {
                        ptr_args.push(ann_arg)
                    } else if anncode == 1 || anncode == 2 {
                        str_args.push(ann_arg)
                    }
                }
                cmd = u32::read_from(f, ByteOrder::LittleEndian)?;
            }
            tickflow.extend(cmd.to_le_bytes());
            let arg_count = (cmd >> 10) & 0xF;
            for i in 0..arg_count {
                let arg = u32::read_from(f, ByteOrder::LittleEndian)?;
                if str_args.contains(&i) {
                    pointers.push(Pointer {
                        offset: tickflow.len() as u32,
                        ptype: PointerType::String,
                    })
                } else if ptr_args.contains(&i) {
                    pointers.push(Pointer {
                        offset: tickflow.len() as u32,
                        ptype: PointerType::Tickflow,
                    })
                }
                tickflow.extend(arg.to_le_bytes());
            }
        }
        let mut strings = vec![0; (file_size - f.stream_position()?) as usize];
        f.read(&mut strings)?;
        let stringpos = tickflow.len();
        //fix string pointers - stringpos, etc
        for ptr in &pointers {
            if let PointerType::String = ptr.ptype {
                let mut ptr_bytes: [u8; 4] = [0; 4];
                for i in 0..4 {
                    ptr_bytes[i] = tickflow[ptr.offset as usize + i];
                }
                let str_ptr = (u32::from_le_bytes(ptr_bytes) - stringpos as u32).to_le_bytes();
                for i in 0..4 {
                    tickflow[ptr.offset as usize + i] = str_ptr[i];
                }
            }
        }

        let section_flow = FlowSection {
            start_offset: start,
            tickflow_data: tickflow,
        };
        let pointers = match pointers.len() {
            0 => None,
            _ => Some(pointers),
        };
        let strings = match strings.len() {
            0 => None,
            _ => Some(strings), // TODO: maybe tickompiler doesn't add the 0xFFFFFFFE
                                // if there's no strings???
        };
        return Ok(Self {
            flow: section_flow,
            ptro: pointers,
            tmpo: None, //TODO: in the future, only make it None if there's no tempos
            strd: strings,
        });
    }

    pub fn to_btks_file<F: Write + Seek>(&self, f: &mut F) -> io::Result<()> {
        // ------------
        //    Header
        // ------------
        f.write(b"BTKS")?; //magic
        let mut size = Self::HEADER_SIZE;
        let mut num_sections = 1;
        let size_pos = f.stream_position()?;
        0u32.write_to(f, ByteOrder::LittleEndian)?;
        Self::REVISION.write_to(f, ByteOrder::LittleEndian)?;
        let num_sections_pos = f.stream_position()?;
        0u32.write_to(f, ByteOrder::LittleEndian)?;

        // ----------
        //    FLOW
        // ----------
        f.write(b"FLOW")?; //magic
        let flow_size = Self::FLOW_HEADER + self.flow.tickflow_data.len() as u32;
        size += flow_size;
        flow_size.write_to(f, ByteOrder::LittleEndian)?;
        self.flow
            .start_offset
            .write_to(f, ByteOrder::LittleEndian)?;
        f.write(&self.flow.tickflow_data)?;

        // ----------
        //    PTRO
        // ----------
        if let Some(c) = &self.ptro {
            num_sections += 1;
            f.write(b"PTRO")?; //magic
            let ptro_size: u32 = Self::PTRO_HEADER + c.len() as u32 * 5;
            size += ptro_size;
            ptro_size.write_to(f, ByteOrder::LittleEndian)?;
            (c.len() as u32).write_to(f, ByteOrder::LittleEndian)?;
            for pointer in c {
                f.write(&pointer.to_bin())?;
            }
        }

        //TODO: tmpo

        // ----------
        //    STRD
        // ----------
        if let Some(c) = &self.strd {
            num_sections += 1;
            f.write(b"STRD")?; //magic
            let strd_size: u32 = Self::STRD_HEADER + c.len() as u32;
            size += strd_size;
            strd_size.write_to(f, ByteOrder::LittleEndian)?;
            f.write(&c)?;
        }

        // Write filesize and number of sections
        f.seek(SeekFrom::Start(size_pos))?;
        size.write_to(f, ByteOrder::LittleEndian)?;
        f.seek(SeekFrom::Start(num_sections_pos))?;
        num_sections.write_to(f, ByteOrder::LittleEndian)?;

        Ok(())
    }
}
