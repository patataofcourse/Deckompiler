#[derive(Debug, Clone)]
pub struct Tempo {
    pub id: u32,
    pub data: Vec<TempoVal>,
}

#[derive(Debug, Clone)]
pub struct TempoVal {
    pub beats: u32,
    pub time: u32, // in 32000ths of a second
    pub loop_val: u32,
}
