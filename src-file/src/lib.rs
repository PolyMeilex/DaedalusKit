use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

// Ignoring non-utf8 strings for file paths, because it's way to obnocious to handle correctly
// cross-platform
pub fn lines(src: &str) -> impl Iterator<Item = PathBuf> + '_ {
    src.lines()
        .map(|line| line.split("//").next().unwrap())
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut path = PathBuf::new();
            for elem in line.split('\\') {
                path.push(elem);
            }
            path
        })
}

pub fn load(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let path = path.as_ref();
    let base = path.parent().unwrap();

    let mut out = Vec::new();

    let src = std::fs::read_to_string(path).unwrap();

    let lines = src
        .lines()
        .map(|line| line.split("//").next().unwrap())
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty());

    for line in lines {
        resolve(base, line, &mut out);
    }

    out
}

pub fn resolve(base: &Path, line: &str, out: &mut Vec<PathBuf>) {
    let mut path = base.to_path_buf();

    for elem in line.split('\\') {
        path.push(elem);
    }

    let extension = path.extension().unwrap();
    let file = path.file_name().unwrap().to_str().unwrap();

    if file.contains('*') && extension == OsStr::new("d") {
        let mut res: Vec<_> = glob::glob(path.to_str().unwrap())
            .unwrap()
            .map(|path| path.unwrap())
            .collect();

        // Windows-like sorting
        res.sort_by(|a, b| {
            let a = a.file_name().unwrap().to_str().unwrap().to_lowercase();
            let b = b.file_name().unwrap().to_str().unwrap().to_lowercase();
            if a.starts_with(&b) {
                if a.len() > b.len() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            } else {
                a.cmp(&b)
            }
        });

        for path in res {
            out.push(path);
        }
    } else {
        out.push(path);
    }
}
