use std::{
    error::Error,
    io::{Cursor, Write},
};

use dat_file::text;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let (input, reserved) = match args.as_slice() {
        [input] => (*input, true),
        ["--no-reserved", input] | [input, "--no-reserved"] => (*input, false),
        _ => return Err("usage: dat_print [--no-reserved] <file.DAT>".into()),
    };

    let bytes = std::fs::read(input).map_err(|e| format!("failed to read {input}: {e}"))?;
    let dat = dat_file::DatFile::decode(&mut Cursor::new(bytes))
        .map_err(|e| format!("failed to decode {input}: {e}"))?;

    let text = text::print(&dat, reserved);

    std::io::stdout().lock().write_all(text.as_bytes()).ok();

    Ok(())
}
