use pretty_assertions::assert_eq;
use std::io::Cursor;

fn main() {
    let mut args = std::env::args().skip(1);

    let a = args.next().expect("Arg `a` not found");
    let b = args.next().expect("Arg `b` not found");

    let data = std::fs::read(a).unwrap();
    let dat_a = dat_file::DatFile::decode(&mut Cursor::new(data)).unwrap();
    // dat_file::debug_print(&dat_a);

    let data = std::fs::read(b).unwrap();
    let dat_b = dat_file::DatFile::decode(&mut Cursor::new(data)).unwrap();
    // dat_file::debug_print(&dat_b);

    assert_eq!(dat_a.symbols.len(), dat_b.symbols.len());

    for (a, b) in dat_a.symbols.iter().zip(dat_b.symbols.iter()) {
        assert_eq!(a, b);
    }

    assert_eq!(
        dat_a.bytecode.as_bytes().len(),
        dat_b.bytecode.as_bytes().len()
    );

    for (a, b) in dat_a
        .bytecode
        .instructions()
        .zip(dat_b.bytecode.instructions())
    {
        assert_eq!(a, b);
    }
}
