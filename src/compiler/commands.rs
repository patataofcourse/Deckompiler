use std::ops::RangeInclusive;

use tickflow_parse::old::ParsedValue;

#[derive(Clone)]
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

    pub(crate) fn calc_arg_range(&self) -> RangeInclusive<usize> {
        let end = self.args.len();
        let start = self
            .args
            .iter()
            .enumerate()
            .find(|(_, c)| matches!(c, Opt(_)))
            .map(|c| c.0)
            .unwrap_or(end);
        start..=end
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Arg0Type {
    Set(u32),
    Argument,
    Any,
}

pub fn resolve_command(
    cmd: &String,
    arg0: Option<u32>,
    mut args: Vec<ParsedValue>,
) -> Result<(u16, u32, Vec<ParsedValue>), ResolveError> {
    let mut out = None;
    let mut is_arg0d = false;
    for def in TICKOMPILER_COMMANDS {
        if cmd == def.0 {
            match def.1.arg0 {
                Set(_) if arg0.is_some() => Err(ResolveError::Arg0IsSet(cmd.clone()))?,
                Set(c) => out = Some((def.1.clone(), c)),
                Argument => {
                    out = {
                        is_arg0d = true;
                        if args.is_empty() {
                            Err(ResolveError::WrongArgCount {
                                cmd: cmd.clone(),
                                expected: def.1.calc_arg_range(),
                                got: 0,
                            })?
                        }
                        let c = args.remove(0);
                        let ParsedValue::Integer(arg0) = c else {
                            Err(ResolveError::WrongArgType {
                                cmd: cmd.clone(),
                                arg: 0,
                                expected: "integer".to_string(),
                                got: match c {
                                    ParsedValue::Label(_) => "label/loc",
                                    ParsedValue::String {
                                        is_unicode: false, ..
                                    } => "string",
                                    ParsedValue::String {
                                        is_unicode: true, ..
                                    } => "unicode string",
                                    ParsedValue::Integer(_) => unreachable!(),
                                }
                                .to_string(),
                            })?
                        };
                        Some((def.1.clone(), arg0 as u32))
                    }
                }
                Any => out = Some((def.1.clone(), arg0.unwrap_or(0))),
            }
            break;
        }
    }
    match out {
        Some((def, arg0)) => {
            if def.arg0 == Any && arg0 != 0 {
                // skip arg checker if non-standard arg0
                return Ok((def.num, arg0, args));
            }
            let rng = def.calc_arg_range();
            if !rng.contains(&args.len()) {
                Err(ResolveError::WrongArgCount {
                    cmd: cmd.clone(),
                    expected: rng,
                    got: args.len(),
                })?
            }
            let mut new_args = vec![];
            for (i, arg) in args.into_iter().enumerate() {
                match &arg {
                    ParsedValue::Integer(_) => {
                        if matches!(def.args[i], Int | Opt(_)) {
                            new_args.push(arg)
                        } else {
                            Err(ResolveError::WrongArgType {
                                cmd: cmd.clone(),
                                arg: i + is_arg0d as usize,
                                expected: {
                                    match def.args[i] {
                                        Label => "label",
                                        c if c == AString => "string",
                                        c if c == UString => "unicode string",
                                        _ => unreachable!(),
                                    }
                                    .to_string()
                                },
                                got: "integer".to_string(),
                            })?
                        }
                    }
                    ParsedValue::String { is_unicode, .. } => {
                        if matches!(def.args[i], c if (c == UString) == *is_unicode) {
                            new_args.push(arg)
                        } else {
                            Err(ResolveError::WrongArgType {
                                cmd: cmd.clone(),
                                arg: i + is_arg0d as usize,
                                expected: {
                                    match def.args[i] {
                                        Int | Opt(_) => "integer",
                                        Label => "label",
                                        _ => unreachable!(),
                                    }
                                    .to_string()
                                },
                                got: if *is_unicode {
                                    "unicode string"
                                } else {
                                    "string"
                                }
                                .to_string(),
                            })?
                        }
                    }
                    ParsedValue::Label(_) => {
                        if matches!(def.args[i], Label) {
                            new_args.push(arg)
                        } else {
                            Err(ResolveError::WrongArgType {
                                cmd: cmd.clone(),
                                arg: i + is_arg0d as usize,
                                expected: {
                                    match def.args[i] {
                                        Int | Opt(_) => "integer",
                                        c if c == AString => "string",
                                        c if c == UString => "unicode string",
                                        _ => unreachable!(),
                                    }
                                    .to_string()
                                },
                                got: "label".to_string(),
                            })?
                        }
                    }
                }
            }
            if new_args.len() < def.args.len() {
                for arg in def.args.iter().skip(new_args.len()) {
                    let Opt(v) = arg else { unreachable!() };
                    new_args.push(ParsedValue::Integer(*v));
                }
            }
            assert_eq!(new_args.len(), def.args.len());
            Ok((def.num, arg0, new_args))
        }
        None => Err(ResolveError::Undefined(cmd.clone()))?,
    }
}

pub enum ResolveError {
    WrongArgCount {
        cmd: String,
        expected: RangeInclusive<usize>,
        got: usize,
    },
    WrongArgType {
        cmd: String,
        arg: usize,
        expected: String,
        got: String,
    },
    Arg0IsSet(String),
    Undefined(String),
}

impl From<ResolveError> for std::io::Error {
    fn from(value: ResolveError) -> Self {
        Self::new(
            std::io::ErrorKind::Other,
            match value {
                ResolveError::WrongArgCount { cmd, expected, got } => format!(
                    "Command {cmd} takes {}-{} arguments, but {got} were given",
                    expected.start(),
                    expected.end()
                ),
                ResolveError::WrongArgType {
                    cmd,
                    arg,
                    expected,
                    got,
                } => format!(
                    "Command {cmd}'s argument #{} is of type '{expected}', but '{got}' was given",
                    arg
                ),
                ResolveError::Arg0IsSet(cmd) => {
                    format!("Command {cmd} has a predefined Arg0, so it can't be manually given")
                }
                ResolveError::Undefined(cmd) => format!("Command {cmd} not found"),
            },
        )
    }
}

use Arg0Type::*;
use ArgType::{Int, Label, Opt};

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
    CmdDef::named("getrest", 0xf, Set(0), &[Int]),
    CmdDef::named("setrest", 0xf, Set(1), &[Int, Int]),
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
    CmdDef::named("random", 0xB8, Argument, &[]),
];
