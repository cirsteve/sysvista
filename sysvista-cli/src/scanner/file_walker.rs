use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct WalkedFile {
    pub path: PathBuf,
    pub relative_path: String,
}

pub fn walk_directory(root: &Path) -> (Vec<WalkedFile>, u64) {
    let mut files = Vec::new();
    let mut skipped: u64 = 0;

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                    continue;
                }
                let path = entry.path().to_path_buf();
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                files.push(WalkedFile {
                    path,
                    relative_path: relative,
                });
            }
            Err(_) => {
                skipped += 1;
            }
        }
    }

    (files, skipped)
}
