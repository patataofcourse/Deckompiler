use std::str::FromStr;

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
    const LOOP_VAL_DEFAULT: u32 = 2;

    pub fn from_tickompiler_file(tempo: String) -> Option<Self> {
        let mut lines = tempo.lines();
        let id = match u32::from_str_radix(lines.next()?, 16) {
            Ok(c) => c,
            Err(_) => return None,
        };
        let mut data = vec![];

        for line in lines {
            let (tempo_str, line) = line.split_once(" ")?;
            let tempo = match f32::from_str(tempo_str) {
                Ok(c) => c,
                Err(_) => return None,
            };
            let (beats_str, line) = match line.split_once(" ") {
                Some(c) => c,
                None => (line, ""),
            };
            let beats = match f32::from_str(beats_str) {
                Ok(c) => c,
                Err(_) => return None,
            };
            let loop_val = if line == "" {
                Self::LOOP_VAL_DEFAULT
            } else {
                let (loop_str, line) = match line.split_once(" ") {
                    Some(c) => c,
                    None => (line, ""),
                };
                match u32::from_str(loop_str) {
                    Ok(c) => c,
                    Err(_) => return None,
                }
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
