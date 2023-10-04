pub struct CmdDef {
    pub num: u16,
    pub arg0: Arg0Type,
    pub args: &'static [ArgType],
}

impl CmdDef {
    pub const fn new(num: u16, arg0: Arg0Type, args: &'static [ArgType]) -> Self {
        Self { num, arg0, args }
    }

    pub const fn named(
        name: &'static str,
        num: u16,
        arg0: Arg0Type,
        args: &'static [ArgType],
    ) -> (&'static str, Self) {
        (name, Self::new(num, arg0, args))
    }
}

pub enum ArgType {
    Int,
    String(bool),
    Label,
    Opt(i32),
}

#[allow(non_upper_case_globals)]
pub const AString: ArgType = ArgType::String(false);
#[allow(non_upper_case_globals)]
pub const UString: ArgType = ArgType::String(true);

pub enum Arg0Type {
    Set(u32),
    Argument,
    Any,
}

use Arg0Type::*;
use ArgType::*;

// a bunch of these commands have arg0 variations
// remove argument checking for those
#[rustfmt::skip]
pub const TICKOMPILER_COMMANDS: &[(&str, CmdDef)] = &[
    CmdDef::named("async_sub", 0, Any, &[Int, Opt(0), Opt(2000)]),
    CmdDef::named("get_async", 1, Set(0), &[Int, Opt(0)]),
    CmdDef::named("set_func", 1, Set(1), &[Int, Label]),
    CmdDef::named("async_call", 2, Any, &[Label, Opt(0)]),
    CmdDef::named("kill_all", 3, Set(0), &[]),
    CmdDef::named("kill_cat", 3, Set(1), &[Int]),
    CmdDef::named("kill_loc", 3, Set(2), &[Label]),
    CmdDef::named("kill_sub", 3, Set(3), &[Int]),
    CmdDef::named("sub", 4, Any, &[Int]),
    CmdDef::named("get_sync", 5, Any, &[Int]),
    CmdDef::named("call", 6, Any, &[Label]),
    CmdDef::named("return", 7, Any, &[]),
    CmdDef::named("stop", 8, Any, &[]),
    CmdDef::named("set_cat", 9, Any, &[Int]),
    CmdDef::named("set_condvar", 0xa, Any, &[Int]),
    CmdDef::named("add_condvar", 0xb, Any, &[Int]),
    CmdDef::named("push_condvar", 0xc, Any, &[]),
    CmdDef::named("pop_condvar", 0xd, Any, &[]),
    CmdDef::named("rest", 0xe, Argument, &[]),
    CmdDef::named("setrest", 0xf, Set(0), &[Int, Int]),
    CmdDef::named("getrest", 0xf, Set(1), &[Int]),
    CmdDef::named("rest_reset", 0x11, Any, &[]),
    CmdDef::named("unrest", 0x12, Argument, &[]),
    CmdDef::named("label", 0x14, Argument, &[]),
    CmdDef::named("goto", 0x15, Argument, &[]),
    CmdDef::named("if", 0x16, Set(0), &[Int]),
    CmdDef::named("if_neq", 0x16, Set(1), &[Int]),
    CmdDef::named("if_lt", 0x16, Set(2), &[Int]),
    CmdDef::named("if_leq", 0x16, Set(3), &[Int]),
    CmdDef::named("if_gt", 0x16, Set(4), &[Int]),
    CmdDef::named("if_geq", 0x16, Set(5), &[Int]),
    CmdDef::named("else", 0x17, Any, &[]),
    CmdDef::named("endif", 0x18, Any, &[]),
    CmdDef::named("switch", 0x19, Any, &[]),
    CmdDef::named("case", 0x1A, Argument, &[]),
    CmdDef::named("break", 0x1B, Any, &[]),
    CmdDef::named("default", 0x1C, Any, &[]),
    CmdDef::named("endswitch", 0x1D, Any, &[]),
    CmdDef::named("set_countdown", 0x1E, Set(0), &[Int]),
    CmdDef::named("set_countdown_condvar", 0x1E, Set(1), &[]),
    CmdDef::named("get_countdown_init", 0x1E, Set(2), &[]),
    CmdDef::named("get_countdown_prog", 0x1E, Set(3), &[]),
    CmdDef::named("get_countdown", 0x1E, Set(4), &[]),
    CmdDef::named("dec_countdown", 0x1E, Set(5), &[]),
    CmdDef::named("speed", 0x24, Any, &[Int]),
    CmdDef::named("speed_relative", 0x25, Any, &[Int, Int, Int]),
    CmdDef::named("engine", 0x28, Any, &[Int]),
    CmdDef::named("game_model", 0x2A, Set(0), &[Int, Int]),
    CmdDef::named("game_cellanim", 0x2A, Set(2), &[Int, Int]),
    CmdDef::named("game_effect", 0x2A, Set(3), &[Int, Int]),
    CmdDef::named("game_layout", 0x2A, Set(4), &[Int, Int]),
    CmdDef::named("set_model", 0x31, Set(0), &[Int, UString, Opt(1)]),
    CmdDef::named("remove_model", 0x31, Set(1), &[Int]),
    CmdDef::named("has_model", 0x31, Set(2), &[Int]),
    CmdDef::named("set_cellanim", 0x35, Set(0), &[Int, UString, Opt(-1)]),
    CmdDef::named("cellanim_busy", 0x35, Set(1), &[Int]),
    CmdDef::named("remove_cellanim", 0x35, Set(3), &[Int]),
    CmdDef::named("set_effect", 0x39, Set(0), &[Int, UString, Opt(-1)]),
    CmdDef::named("effect_busy", 0x39, Set(1), &[Int]),
    CmdDef::named("remove_effect", 0x39, Set(7), &[Int]),
    CmdDef::named("set_layout", 0x3E, Set(0), &[Int, UString, Opt(-1)]),
    CmdDef::named("layout_busy", 0x3E, Set(1), &[Int]),
    CmdDef::named("remove_layout", 0x3E, Set(7), &[Int]),
    CmdDef::named("play_sfx", 0x40, Any, &[Int]),
    CmdDef::named("set_sfx", 0x5D, Any, &[Int, UString]),
    CmdDef::named("remove_sfx", 0x5F, Any, &[Int]),
    CmdDef::named("input", 0x6A, Any, &[Int]),
    CmdDef::named("fade", 0x7D, Any, &[Int, Int, Int]),
    CmdDef::named("zoom", 0x7E, Set(0), &[Int, Int, Int]),
    CmdDef::named("zoom_gradual", 0x7E, Set(1), &[Int, Int, Int, Int, Int, Int]),
    CmdDef::named("pan", 0x7F, Set(0), &[Int, Int, Int]),
    CmdDef::named("pan_gradual", 0x7F, Set(1), &[Int, Int, Int, Int, Int, Int]),
    CmdDef::named("rotate", 0x80, Set(0), &[Int, Int]),
    CmdDef::named("rotate_gradual", 0x80, Set(1), &[Int, Int, Int, Int, Int]),
    CmdDef::named("star", 0xAE, Any, &[Int]),
    CmdDef::named("debug", 0xB5, Any, &[AString]),
];
