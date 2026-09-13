use std::io::Cursor;

use daedalus_bytecode::Instruction;
use dat_file::{DatFile, Symbol, SymbolData};

fn main() {
    // /home/poly/Gothic2/_work/Data/Scripts/_compiled/CAMERA.DAT
    let path = std::env::args().nth(1).unwrap();
    let src = std::fs::read(path).unwrap();

    let decoded = dat_file::DatFile::decode(&mut Cursor::new(&src)).unwrap();
    // dat_file::debug_print(&decoded);

    println!("{decoded:#?}");
}
