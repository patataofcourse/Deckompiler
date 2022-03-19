use crate::common::Tempo;
use std::io::{Read, Result as IOResult, Seek};

#[derive(Debug)]
pub struct C00Bin {
    pub c00_type: C00Type,
    pub base_patch: Patch,
    pub tickflows: Vec<TickompilerBinary>,
    pub tempos: Vec<Tempo>,
}

#[derive(Debug, Clone)] //TODO: IPS??? custom format??? to be decided
pub struct Patch;

#[derive(Debug, Clone)]
pub enum C00Type {
    RHMPatch,
    SaltwaterUS,
    SaltwaterEU,
    SaltwaterJP,
    SaltwaterKR,
}

#[derive(Debug, Clone)]
pub struct TickompilerBinary {
    pub index: u32,
    pub start: u32,
    pub assets: u32,
    pub data: Vec<u8>,
}

impl C00Bin {
    pub fn from_file<F: Read + Seek>(file: &mut F) -> IOResult<Self> {
        unimplemented!();
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
