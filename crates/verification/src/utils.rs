use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Result;
use sha2::{Digest, Sha256};

pub(crate) fn read_files<P>(path: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mut files = Vec::new();

    if path.exists() {
        for entry in path.read_dir()?.flatten() {
            if entry.metadata()?.is_dir() {
                files.extend(read_files(entry.path())?);
            }
            if entry.metadata()?.is_file() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if file_name_str != super::SIGNATURE
                    && !super::EXCLUDES.iter().any(|&s| s == file_name_str.as_ref())
                {
                    files.push(entry.path());
                }
            }
        }
    }

    Ok(files)
}

pub(crate) fn get_hash<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let mut buf = [0; 1024];
    let mut file = fs::OpenOptions::new().read(true).open(path.as_ref())?;
    let mut hash = Sha256::new();
    let mut n;

    loop {
        n = file.read(&mut buf)?;

        if n == 0 {
            break;
        }

        hash.update(buf);
    }
    Ok(hash.finalize().to_vec())
}
