pub struct BTKS(Vec<Section>);

impl BTKS {
    const REVISION: u32 = 0;
}

pub enum Section {
    FlowSection {
        start_offset: u32,
        tickflow_data: Vec<u8>,
    },
    PtroSection (Vec<Pointer>),
    TmpoSection (Vec<Tempo>),
    StrdSection (Vec<u8>)
}

pub struct Pointer {
    offset: u32,
    ptype: PointerType,
}

#[repr(u8)]
pub enum PointerType {
    String,
    Tickflow,
}

pub struct Tempo {
    id: u32,
    data: Vec<TempoVal>,
}

pub struct TempoVal {
    beats: u32,
    time: u32, //NOT SECONDS
    loop_val: u32,
}