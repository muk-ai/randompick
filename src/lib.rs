use rand::seq::SliceRandom;
use rand::thread_rng;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub fn random_pick(path: &Path) -> Option<PathBuf> {
    let options = allowd_extensions_from_args();
    let files = get_files(path, &options).unwrap();
    pick_one(files)
}

fn get_files(
    path: &Path,
    allowd_extensions: &Option<Vec<OsString>>,
) -> std::io::Result<Vec<PathBuf>> {
    let dir = fs::read_dir(path)?;
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in dir {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let mut dir_files = get_files(&entry.path(), allowd_extensions)?;
            files.append(&mut dir_files);
        } else if file_type.is_file() {
            if let Some(extensions) = allowd_extensions {
                if let Some(extension) = entry.path().extension() {
                    if extensions.contains(&extension.to_os_string()) {
                        files.push(entry.path());
                    }
                }
            } else {
                files.push(entry.path());
            }
        } else if file_type.is_symlink() {
            // ignore
        } else {
            panic!(
                "This file is neither a directory nor a file. {:?}",
                entry.path()
            )
        }
    }
    Ok(files)
}

fn pick_one(files: Vec<PathBuf>) -> Option<PathBuf> {
    let mut rng = thread_rng();
    return files.choose(&mut rng).cloned();
}

fn allowd_extensions_from_args() -> Option<Vec<OsString>> {
    let args: Vec<String> = std::env::args().into_iter().skip(1).collect();
    let mut extensions: Vec<OsString> = vec![];
    for chunk in args.chunks(2) {
        if let [name, value] = chunk {
            if name == "-e" {
                extensions.push(OsString::from(value))
            }
        }
    }
    if extensions.is_empty() {
        None
    } else {
        Some(extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn get_files_recursively() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/three-tiered_directory");
        let files = get_files(&path, &None).unwrap();
        let mut files: Vec<&OsStr> = files
            .iter()
            .filter_map(|path| path.as_path().file_name())
            .collect();
        files.sort();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], OsStr::new("file1_in_dir1"));
        assert_eq!(files[1], OsStr::new("file2_in_dir2"));
        assert_eq!(files[2], OsStr::new("file3_in_dir3"));
    }

    #[test]
    fn ignore_symlink() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/symlink");
        let files = get_files(&path, &None).unwrap();
        let files: Vec<&OsStr> = files
            .iter()
            .filter_map(|path| path.as_path().file_name())
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], OsStr::new("file1"));
    }

    #[test]
    fn filter_by_markdown_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let extensions = Some(vec![OsString::from("md")]);
        path.push("tests/filter_by_markdown_file");
        let files = get_files(&path, &extensions).unwrap();
        let files: Vec<&OsStr> = files
            .iter()
            .filter_map(|path| path.as_path().file_name())
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], OsStr::new("file1.md"));
    }
}
