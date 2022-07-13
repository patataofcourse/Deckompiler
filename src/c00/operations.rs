#[derive(Debug)]
pub struct TickflowOp {
    pub is_unicode: bool, // To be used with string operations
    pub command: u32,     // Following the tickflow bytecode specifications
    pub args: Vec<u8>,
    pub scene: Option<u32>,
}

const OPCODE_MASK: u32 = 0xFFFFC3FF;

#[macro_export]
macro_rules! tf_op_args {
    ($cmdname:literal $(<$arg0:literal>)?, $args:expr $(, $scene:literal)? $(, is_unicode=$is_unicode:literal)? $(,)?) => {
        {
        let command = ($cmdname & 0x3FF) $(+ ($arg0 << 14))?;

        #[allow(unused_mut, unused_assignments)]
        let mut is_unicode = false;
        $(is_unicode = $is_unicode;)?

        #[allow(unused_mut, unused_assignments)]
        let mut scene = None;
        $(scene = Some($scene);)?

        TickflowOp {
            is_unicode,
            command,
            args: $args,
            scene,
            }
        }
    };
}

#[macro_export]
macro_rules! tf_op {
    ($cmdname:literal $(<$arg0:literal>)?) => {
        {
            let command = ($cmdname & 0x3FF) $(+ ($arg0 << 14))?;
            TickflowOp {
                is_unicode: false,
                command,
                args: vec![],
                scene: None,
            }
        }
    };
}

pub fn string_ops() -> Vec<TickflowOp> {
    vec![
        tf_op_args!(0x31, vec![1], is_unicode = true),
        tf_op_args!(0x35, vec![1], is_unicode = true),
        tf_op_args!(0x39, vec![1], is_unicode = true),
        tf_op_args!(0x3B, vec![2]),
        tf_op_args!(0x3E, vec![1], is_unicode = true),
        tf_op_args!(0x5D, vec![1], is_unicode = true),
        tf_op_args!(0x5D<2>, vec![0], is_unicode = true),
        tf_op_args!(0x61<2>, vec![0], is_unicode = true),
        tf_op_args!(0x65<1>, vec![1]),
        tf_op_args!(0x66, vec![1]),
        tf_op_args!(0x67<1>, vec![1]),
        tf_op_args!(0x68<1>, vec![1]),
        tf_op_args!(0x93, vec![2, 3]),
        tf_op_args!(0x94, vec![1, 2, 3]),
        tf_op_args!(0x95, vec![1]),
        tf_op_args!(0xAF<2>, vec![2]),
        tf_op_args!(0xB0<4>, vec![1]),
        tf_op_args!(0xB0<5>, vec![1]),
        tf_op_args!(0xB0<6>, vec![1]),
        tf_op_args!(0xB5, vec![0]),
        tf_op_args!(0x105, vec![0], 1),
        tf_op_args!(0x107, vec![0], 0xC),
        tf_op_args!(0x107<1>, vec![0], 0xC),
        tf_op_args!(0x106, vec![0], 0x18),
        tf_op_args!(0x106, vec![0], 0x2A),
        tf_op_args!(0x10B, vec![0], 0x2C),
        tf_op_args!(0x107, vec![0], 0x39),
        tf_op_args!(0x107<1>, vec![0], 0x39),
        tf_op_args!(0x108, vec![0], 0x39),
        tf_op_args!(0x109, vec![0, 1], 0x39),
        tf_op_args!(0x10A, vec![0], 0x39),
    ]
}

pub fn is_string_op(opcode: u32, scene: u32) -> Option<TickflowOp> {
    let opcode = opcode & OPCODE_MASK;
    for op in string_ops() {
        if op.command == opcode {
            if match op.scene {
                Some(c) => c == scene,
                None => true,
            } {
                return Some(op);
            }
        }
    }
    None
}

pub const SCENE_OP: u32 = 0x28; //no arg0

pub fn is_scene_op(op: u32) -> bool {
    if SCENE_OP == op & OPCODE_MASK {
        true
    } else {
        false
    }
}

pub fn call_ops() -> Vec<TickflowOp> {
    vec![
        tf_op_args!(0x1<1>, vec![1]),
        tf_op_args!(0x2, vec![0]),
        tf_op_args!(0x3<2>, vec![0]),
        tf_op_args!(0x6, vec![0]),
    ]
}

pub fn is_call_op(opcode: u32) -> Option<TickflowOp> {
    let opcode = opcode & OPCODE_MASK;
    for op in call_ops() {
        if op.command == opcode {
            return Some(op);
        }
    }
    None
}

pub fn depth_ops() -> Vec<TickflowOp> {
    vec![
        tf_op!(0x16),
        tf_op!(0x16<1>),
        tf_op!(0x16<2>),
        tf_op!(0x16<3>),
        tf_op!(0x16<4>),
        tf_op!(0x16<5>),
        tf_op!(0x19),
    ]
}

pub fn is_depth_op(opcode: u32) -> Option<TickflowOp> {
    let opcode = opcode & OPCODE_MASK;
    for op in depth_ops() {
        if op.command == opcode {
            return Some(op);
        }
    }
    None
}

pub fn undepth_ops() -> Vec<TickflowOp> {
    vec![tf_op!(0x18), tf_op!(0x1D)]
}

pub fn is_undepth_op(opcode: u32) -> Option<TickflowOp> {
    let opcode = opcode & OPCODE_MASK;
    for op in undepth_ops() {
        if op.command == opcode {
            return Some(op);
        }
    }
    None
}

pub fn return_ops() -> Vec<TickflowOp> {
    vec![tf_op!(0x7), tf_op!(0x8)]
}

pub fn is_return_op(opcode: u32) -> Option<TickflowOp> {
    let opcode = opcode & OPCODE_MASK;
    for op in return_ops() {
        if op.command == opcode {
            return Some(op);
        }
    }
    None
}
