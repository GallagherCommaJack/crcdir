use crc32fast::Hasher;
use failure::*;
use filebuffer::FileBuffer;
use rayon::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

pub fn hash_file(path: &Path) -> Result<u32, Error> {
    let mut hasher = Hasher::new();
    let fbuffer = FileBuffer::open(path)?;
    let len = fbuffer.len();
    fbuffer.prefetch(0, len);
    hasher.update(
        path.to_str()
            .ok_or(format_err!("path couldn't be read as str: {:?}", path))?
            .as_bytes(),
    );
    hasher.update(&fbuffer);
    Ok(hasher.finalize())
}

pub fn sum_dir(path: WalkDir) -> Result<u32, Error> {
    path.into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .par_bridge()
        .map(|e| hash_file(e.path()))
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)))
}

pub fn hash_dir(path: WalkDir) -> Result<u32, Error> {
    let mut hasher = Hasher::new();

    for entry in path
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
    {
        let path = entry.path();
        let fbuffer = FileBuffer::open(path)?;
        let len = fbuffer.len();
        fbuffer.prefetch(0, len);
        hasher.update(
            path.to_str()
                .ok_or(format_err!("path couldn't be read as str: {:?}", path))?
                .as_bytes(),
        );
        hasher.update(&fbuffer);
    }
    Ok(hasher.finalize())
}
