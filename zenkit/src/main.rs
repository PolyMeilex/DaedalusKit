use zenkit::CutsceneLibrary;

fn main() {
    let cl =
        CutsceneLibrary::load_path("/home/poly/Gothic2/_work/Data/Scripts/Content/CUTSCENE/OU.BIN");

    let count = cl.block_count();
    dbg!(count);

    for id in 0..count {
        let block = cl.block_by_index(id).unwrap();
        dbg!(block.name());

        let msg = block.message();

        dbg!(msg.ty());
        dbg!(msg.text());
        dbg!(msg.name());
    }
}
