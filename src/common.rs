#[derive(Debug, Clone)]
pub struct Tempo {
    id: u32,
    data: Vec<TempoVal>,
}

#[derive(Debug, Clone)]
pub struct TempoVal {
    beats: u32,
    time: u32, //NOT SECONDS
    loop_val: u32,
}
