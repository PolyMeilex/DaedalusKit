use lexer::DaedalusLexer;

#[test]
fn parser_g2notr_diff() {
    let base_path = "../test_data/G2MDK-PolishScripts/Content/";
    let mut src = src_file::load(format!("{base_path}Gothic.src"));
    src.append(&mut src_file::load(format!("{base_path}Fight.src")));

    let len = src.len();

    for (id, path) in src.into_iter().enumerate() {
        let bytes = std::fs::read(&path).unwrap();
        let path = path.strip_prefix(base_path).unwrap();
        println!("{path:?} ({} / {len})", id + 1);

        let (src, _, _) = encoding_rs::WINDOWS_1250.decode(&bytes);
        let mut lexer = DaedalusLexer::new(&src);
        parser::parse::File::parse(&mut lexer).unwrap();
    }
}
