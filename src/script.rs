use clap::App;
use crc32fast::Hasher;
use std::{
    ffi::OsStr,
    fs::{File, FileType},
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Clone, KsonRep)]
pub struct EntityMeta {
    pub def_dir: String,
    pub runfile: String,
    pub checksum: u32,
}

fn hash_dir(path: WalkDir) -> Option<u32> {
    let mut empty = true;
    let mut hasher = Hasher::new();
    for entry in path
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
    {
        empty = false;
        let path = entry.path();
        hasher.update(path.to_str()?.as_bytes());
        let mut buf = vec![];
        let mut file = File::open(path).ok()?;
        file.read_to_end(&mut buf).ok()?;
        hasher.update(&buf);
    }
    if empty {
        None
    } else {
        Some(hasher.finalize())
    }
}

fn main() {}
