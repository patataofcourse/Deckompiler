use bytestream::{ByteOrder, StreamReader};
use std::io::{self, Read, Seek, Write};

#[derive(Debug)]
pub struct BTKS(Vec<Section>);

impl BTKS {
    const REVISION: u32 = 0;
}

#[derive(Debug)]
pub enum Section {
    FlowSection {
        start_offset: u32,
        tickflow_data: Vec<u8>,
    },
    PtroSection(Vec<Pointer>),
    TmpoSection(Vec<Tempo>),
    StrdSection(Vec<u8>),
}

#[derive(Debug)]
pub struct Pointer {
    offset: u32,
    ptype: PointerType,
}

#[derive(Debug)]
#[repr(u8)]
pub enum PointerType {
    String,
    Tickflow,
}

#[derive(Debug)]
pub struct Tempo {
    id: u32,
    data: Vec<TempoVal>,
}

#[derive(Debug)]
pub struct TempoVal {
    beats: u32,
    time: u32, //NOT SECONDS
    loop_val: u32,
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
                println!("{:?}", ptr_bytes);
                let str_ptr = (u32::from_le_bytes(ptr_bytes) - stringpos as u32).to_le_bytes();
                for i in 0..4 {
                    tickflow[ptr.offset as usize + i] = str_ptr[i];
                }
            }
        }

        let section_flow = Section::FlowSection {
            start_offset: start,
            tickflow_data: tickflow,
        };
        let section_ptro = Section::PtroSection(pointers);
        let section_tmpo = None; //TODO: in the future, only make it None if there's no tempos
        let section_strd = Section::StrdSection(strings);
        let mut btks = Self(vec![section_flow, section_ptro]);
        match section_tmpo {
            Some(c) => btks.0.push(c),
            None => (),
        }
        btks.0.push(section_strd);
        println!("{:#?}", btks);
        return Ok(btks);
    }

    pub fn to_btks_file<F: Write>(&self, f: &mut F) -> io::Result<()> {
        /*
            //header
            outfile.write(header["magic"])
            outfile.write(header["size"].to_bytes(4, "little"))
            outfile.write(header["version"].to_bytes(4, "little"))
            outfile.write(header["section_amt"].to_bytes(4, "little"))

            //flow
            outfile.write(section_flow["magic"])
            outfile.write(section_flow["size"].to_bytes(4, "little"))
            outfile.write(section_flow["start"].to_bytes(4, "little"))
            outfile.write(section_flow["tickflow"])

            //ptro
            outfile.write(section_ptro["magic"])
            outfile.write(section_ptro["size"].to_bytes(4, "little"))
            outfile.write(section_ptro["ptr_amt"].to_bytes(4, "little"))
            outfile.write(section_ptro["pointers"])

            //TODO: tmpo

            //strd
            outfile.write(section_strd["magic"])
            outfile.write(section_strd["size"].to_bytes(4, "little"))
            outfile.write(section_strd["strings"])

            outfile.close()
        */
        unimplemented!();
    }
}
