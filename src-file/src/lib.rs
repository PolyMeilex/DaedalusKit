use bstr::{BStr, ByteSlice};

pub fn lines(src: &BStr) -> impl Iterator<Item = &BStr> {
    src.lines()
        .map(|line| line.split_str("//").next().unwrap())
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty())
        .map(BStr::new)
}

// TODO: Resolver for "*.d"
