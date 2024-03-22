use bstr::{BString, ByteSlice};

fn main() {
    let file = std::fs::read("/home/poly/Gothic2/_work/Data/Scripts/Content/Gothic.src").unwrap();
    let file = BString::new(file);

    let lines = src_file::lines(file.as_bstr());

    for line in lines {
        println!("{line}");
    }
}
