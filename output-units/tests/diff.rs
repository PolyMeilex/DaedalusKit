use zenkit::CutsceneLibrary;

#[test]
fn diff_cu() {
    // let cl =
    //     CutsceneLibrary::load_path("/home/poly/Gothic2/_work/Data/Scripts/Content/CUTSCENE/Ou.csl");
    let cl =
        CutsceneLibrary::load_path("/home/poly/.local/share/Steam/steamapps/common/Gothic II (backup)/_work/Data/Scripts/CONTENT/CUTSCENE/Ou.csl");
    let my_cl = CutsceneLibrary::load_path("../a.txt");

    let my_count = my_cl.block_count();
    let count = cl.block_count();

    assert_eq!(count, my_count);

    let mut failed = Vec::new();
    for id in 0..count {
        let block = cl.block_by_index(id).unwrap();

        let res = (0..my_count).find_map(|my| {
            let my = my_cl.block_by_index(my).unwrap();
            if my.name() == block.name() {
                Some(my)
            } else {
                None
            }
        });

        let my_block = res.unwrap();

        assert_eq!(block.name(), my_block.name());

        if block.message().text() != my_block.message().text() {
            let message = block.message();
            let (text, _, _) = encoding_rs::WINDOWS_1250.decode(message.text().to_bytes());
            let my_message = my_block.message();
            let (my_text, _, _) = encoding_rs::WINDOWS_1250.decode(my_message.text().to_bytes());

            let err = format!("{:?}: {:?} != {:?}", block.name(), text, my_text);

            // println!("{err}");

            failed.push(err);
        }
    }

    if !failed.is_empty() {
        let mut out = String::new();

        for err in failed {
            out += &err;
            out += "\n";
        }

        panic!("{out}");
    }
}
