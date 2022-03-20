pub struct TickflowOp {
    pub is_unicode: bool,
    pub command: u32, // Following the tickflow bytecode specifications
    pub args: Vec<u8>,
    pub scene: Option<u32>,
}

#[macro_export]
macro_rules! tf_op_args {
    ($cmdname:literal $(<$arg0:literal>)?, $args:expr $(, $scene:literal)? $(, is_unicode=$is_unicode:literal)? $(,)?) => {
        {
        let command = $cmdname & 0x3FF $(+ $arg0 << 14)?;

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
        tf_op_args!(0x10B, vec![0], 0x2C),
    ]
}

pub fn is_string_op(op: u32) -> Option<TickflowOp> {
    unimplemented!(); //TODO
}

pub fn scene_op() -> TickflowOp {
    tf_op_args!(0x28, vec![0])
}

pub fn call_ops() -> Vec<TickflowOp> {
    vec![
        tf_op_args!(0x1<1>, vec![1]),
        tf_op_args!(0x2, vec![0]),
        tf_op_args!(0x6, vec![0]),
    ]
}
