use std::io::Cursor;

#[test]
fn diff_gothic2_nor() {
    let src = std::fs::read("../test_data/gothic_g2nor.dat").unwrap();

    let decoded = dat::DatFile::decode(&mut Cursor::new(&src)).unwrap();

    let mut encoded = Vec::with_capacity(src.len());
    decoded.encode(&mut encoded).unwrap();

    assert_eq!(src, encoded);
}
