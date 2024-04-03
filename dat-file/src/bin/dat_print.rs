use std::io::Cursor;

fn main() {
    let mut args = std::env::args().skip(1);

    let a = args.next().expect("Arg `a` not found");

    let data = std::fs::read(a).unwrap();
    let dat_a = dat_file::DatFile::decode(&mut Cursor::new(data)).unwrap();
    dat_file::debug_print(&dat_a);
}
