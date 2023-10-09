use bytestream::{ByteOrder::LittleEndian as LE, StreamWriter};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tickflow_parse::old::{parse_from_text, CommandName, Context, ParsedStatement};

pub mod commands;

pub enum CompiledFileType {
    Tickompiler,
    BTKS,
}

pub fn compile_file(
    in_: impl AsRef<Path>,
    out: impl AsRef<Path>,
    out_filetype: CompiledFileType,
) -> tickflow_parse::Result<()> {
    let cwd = in_.as_ref().parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "invalid path for a file",
    ))?;
    let cmds = Context::parse_file(parse_from_text(&mut File::open(&in_)?)?, |c| {
        let mut cwd = PathBuf::from(cwd);
        cwd.push(c);
        File::open(cwd)
    })?;
    match out_filetype {
        CompiledFileType::Tickompiler => to_btkm(File::create(out)?, cmds).map_err(Into::into),
        CompiledFileType::BTKS => to_btks(File::create(out)?, cmds).map_err(Into::into),
    }
}

fn to_btkm(mut out: File, cmds: Context) -> std::io::Result<()> {
    let mut cmd_size = 0;
    let mut resolved_cmds = vec![];
    for cmd in cmds.parsed_cmds {
        let ParsedStatement::Command { cmd, arg0, args } = cmd else {
            resolved_cmds.push(cmd);
            continue;
        };
        let (cmd, arg0, args) = match cmd {
            CommandName::Raw(c) => (c as u16, arg0.unwrap_or(0), args.clone()),
            CommandName::Named(c) => commands::resolve_command(&c, arg0, args.clone())?,
        };
        cmd_size += 4 * (1 + args.len());
        resolved_cmds.push(ParsedStatement::Command {
            cmd: CommandName::Raw(cmd as i32),
            arg0: Some(arg0),
            args,
        });
    }

    // "header"
    cmds.index.write_to(&mut out, LE)?;
    cmds.start[0]
        .unwrap_or(get_pos_of_label(&resolved_cmds, "start").unwrap())
        .write_to(&mut out, LE)?;
    cmds.start[1]
        .unwrap_or(get_pos_of_label(&resolved_cmds, "assets").unwrap())
        .write_to(&mut out, LE)?;

    let cmds = resolved_cmds
        .iter()
        .filter(|c| matches!(c, ParsedStatement::Command { .. }));
    let mut str_data = vec![];

    for cmd in cmds {
        let ParsedStatement::Command {
            cmd: CommandName::Raw(cmd),
            arg0: Some(arg0),
            args,
        } = cmd
        else {
            unreachable!();
        };

        let cmd = *cmd as u16;

        if args.len() > 15 {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "too many arguments given to a command",
            ))?
        }

        let op_int = (cmd & 0x3FF) as u32 + ((args.len() & 0xF) << 10) as u32 + (arg0 << 14);
        let mut parsed_args = vec![];
        let mut arg_anns: Vec<u32> = vec![];
        for (i, arg) in args.iter().enumerate() {
            match arg {
                tickflow_parse::old::ParsedValue::Integer(c) => parsed_args.push(*c),
                tickflow_parse::old::ParsedValue::Label(lab) => {
                    arg_anns.push((i as u32) << 8);
                    parsed_args.push(get_pos_of_label(&resolved_cmds, lab).ok_or(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Could not find label {lab}"),
                        ),
                    )?)
                }
                tickflow_parse::old::ParsedValue::String { value, is_unicode } => {
                    arg_anns.push(((i as u32) << 8) + if *is_unicode { 1 } else { 2 });
                    parsed_args.push((cmd_size + str_data.len()) as i32);
                    let binary_data = if *is_unicode {
                        let mut out = vec![];
                        for i in value.encode_utf16() {
                            out.extend(i.to_le_bytes())
                        }
                        out.extend(vec![0; if out.len() % 4 == 2 { 2 } else { 4 }]);
                        out
                    } else {
                        let mut out = value.bytes().collect::<Vec<_>>();
                        out.extend(vec![0; 4 - (out.len() % 4)]);
                        out
                    };
                    str_data.extend(binary_data);
                }
            }
        }
        if !arg_anns.is_empty() {
            (-1i32).write_to(&mut out, LE)?;
            (arg_anns.len() as u32).write_to(&mut out, LE)?;
            for ann in arg_anns {
                ann.write_to(&mut out, LE)?;
            }
        }
        op_int.write_to(&mut out, LE)?;
        for arg in parsed_args {
            arg.write_to(&mut out, LE)?;
        }
    }
    (-2i32).write_to(&mut out, LE)?;
    out.write_all(&str_data)?;
    Ok(())
}

fn to_btks(mut out: File, cmds: Context) -> std::io::Result<()> {
    todo!()
}

fn get_pos_of_label(cmds: &[ParsedStatement], name: &str) -> Option<i32> {
    let mut cumulative_len = 0;
    for statement in cmds {
        match statement {
            ParsedStatement::Label(c) => {
                if c == name {
                    return Some(cumulative_len);
                }
            }
            ParsedStatement::Command { args, .. } => cumulative_len += 4 * (1 + args.len() as i32),
        }
    }
    None
}
