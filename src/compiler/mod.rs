use bytestream::{ByteOrder::LittleEndian as LE, StreamWriter};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tickflow_parse::old::{parse_from_text, Context, ParsedStatement};

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
    // "header"
    cmds.index.write_to(&mut out, LE)?;
    cmds.start[0].write_to(&mut out, LE)?;
    cmds.start[1].write_to(&mut out, LE)?;

    let labels = cmds
        .parsed_cmds
        .iter()
        .filter(|c| matches!(c, ParsedStatement::Label(..)));
    let cmds = cmds
        .parsed_cmds
        .iter()
        .filter(|c| matches!(c, ParsedStatement::Command { .. }));

    for cmd in cmds {
        let ParsedStatement::Command { cmd, arg0, args } = cmd else {
            unreachable!()
        };
        let (cmd, arg0, args) = match cmd {
            tickflow_parse::old::CommandName::Raw(c) => {
                (*c as u16, arg0.unwrap_or(0), args.clone())
            }
            tickflow_parse::old::CommandName::Named(c) => {
                commands::resolve_command(c, *arg0, args.clone())?
            }
        };
        //TODO: tickflow-parse should take care of this
        if args.len() > 15 {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "too many arguments given to a command",
            ))?
        }

        let op_int = (cmd & 0x3FF) as u32 + ((args.len() & 0xF) << 10) as u32 + (arg0 << 14);
        todo!("writing arguments with argument annotations (ew)")
    }
    Ok(())
}

fn to_btks(mut out: File, cmds: Context) -> std::io::Result<()> {
    todo!()
}
