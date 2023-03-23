use bytestream::{ByteOrder, StreamWriter};
use std::{
    io::{self, Write},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct Tempo {
    pub id: u32,
    pub data: Vec<TempoVal>,
}

#[derive(Debug, Clone)]
pub struct TempoVal {
    pub beats: f32,
    pub time: u32, // in 32000ths of a second
    pub loop_val: u32,
}

impl Tempo {
    pub fn is_streamed(&self) -> bool {
        //TODO: currently always makes custom tempo IDs AAC since they can't be called for BCGRPs
        !(self.id >= 0x01000101 && self.id <= 0x01000281)
    }
}

impl Tempo {
    const LOOP_VAL_DEFAULT: u32 = 2;

    pub fn from_tickompiler_file(tempo: String) -> Option<Self> {
        let mut lines = tempo.lines();
        let id = match u32::from_str_radix(lines.next()?, 16) {
            Ok(c) => c,
            Err(_) => return None,
        };
        let mut data = vec![];

        for line in lines {
            let (tempo_str, line) = line.split_once(' ')?;
            let tempo = match f32::from_str(tempo_str) {
                Ok(c) => c,
                Err(_) => return None,
            };
            let (beats_str, line) = match line.split_once(' ') {
                Some(c) => c,
                None => (line, ""),
            };
            let beats = f32::from_str(beats_str).ok()?;
            let loop_val = if line.is_empty() {
                Self::LOOP_VAL_DEFAULT
            } else {
                let (loop_str, _) = match line.split_once(' ') {
                    Some(c) => c,
                    None => (line, ""),
                };
                u32::from_str(loop_str).ok()?
            };
            let time: u32 = (beats / tempo * 60.0 * 32000.0) as u32;
            data.push(TempoVal {
                beats,
                time,
                loop_val,
            })
        }

        Some(Self { id, data })
    }
}

impl StreamWriter for Tempo {
    fn write_to<W: Write>(&self, buffer: &mut W, order: ByteOrder) -> io::Result<()> {
        self.id.write_to(buffer, order)?;
        (self.data.len() as u32).write_to(buffer, order)?;
        (if self.is_streamed() { 1u32 } else { 0u32 }).write_to(buffer, order)?;
        for value in &self.data {
            buffer.write_all(&match order {
                ByteOrder::BigEndian => value.beats.to_be_bytes(),
                ByteOrder::LittleEndian => value.beats.to_le_bytes(),
            })?;
            value.time.write_to(buffer, order)?;
            value.loop_val.write_to(buffer, order)?;
        }
        Ok(())
    }
}
