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
    let mut elems: Vec<&str> = line.split('\\').collect();
    let file = elems.pop().unwrap();

    let dir = elems.into_iter().fold(base.to_path_buf(), |dir, elem| {
        join_case_insensitive(&dir, elem)
    });

    let path = if file.contains('*') {
        dir.join(file)
    } else {
        join_case_insensitive(&dir, file)
    };

    let extension = path.extension().unwrap();
    let file = path.file_name().unwrap().to_str().unwrap();

    if file.contains('*') && extension == OsStr::new("d") {
        let options = glob::MatchOptions {
            case_sensitive: false,
            ..Default::default()
        };
        let mut res: Vec<_> = glob::glob_with(path.to_str().unwrap(), options)
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

/// `dir` joined with `name`, or with the entry of `dir` that differs from `name` only in case
fn join_case_insensitive(dir: &Path, name: &str) -> PathBuf {
    let exact = dir.join(name);
    if exact.exists() {
        return exact;
    }

    let entries = std::fs::read_dir(dir).into_iter().flatten().flatten();
    entries
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|entry| entry.eq_ignore_ascii_case(name))
        })
        .unwrap_or(exact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive() {
        let base = std::env::temp_dir().join(format!("src-file-test-{}", std::process::id()));
        std::fs::create_dir_all(base.join("SFX")).unwrap();
        std::fs::write(base.join("SFX/SfxInst.d"), "").unwrap();
        std::fs::write(base.join("SFX/Other.D"), "").unwrap();

        let mut out = Vec::new();
        resolve(&base, "Sfx\\SFXINST.D", &mut out);
        resolve(&base, "sfx\\*.d", &mut out);
        std::fs::remove_dir_all(&base).unwrap();

        let sfx = base.join("SFX");
        assert_eq!(
            out,
            [
                sfx.join("SfxInst.d"),
                sfx.join("Other.D"),
                sfx.join("SfxInst.d")
            ]
        );
    }
}
