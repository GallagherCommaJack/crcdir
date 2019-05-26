use crc32fast::Hasher;
use failure::*;
use filebuffer::FileBuffer;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Hashes an individual file using `crc32`
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

/// Walks a directory, hashing the files and summing the results.
pub fn sum_dir(path: WalkDir) -> Result<u32, Error> {
    path.into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .par_bridge()
        .map(|e| -> Result<(PathBuf, FileBuffer), Error> {
            let path = e.into_path();
            let fbuffer = FileBuffer::open(&path)?;
            let len = fbuffer.len();
            fbuffer.prefetch(0, len);
            Ok((path, fbuffer))
        })
        .filter_map(Result::ok)
        .map(|(path, fbuffer)| {
            let mut hasher = Hasher::new();
            hasher.update(
                path.to_str()
                    .ok_or(format_err!("path couldn't be read as str: {:?}", path))?
                    .as_bytes(),
            );
            hasher.update(&fbuffer);
            Ok(hasher.finalize())
        })
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)))
}

/// Walks a directory, hashing the files sequentially.
pub fn hash_dir(path: WalkDir) -> Result<u32, Error> {
    let mut hasher = Hasher::new();

    let mut fbuffers = Vec::new();
    for entry in path
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
    {
        let path = entry.into_path();
        let fbuffer = FileBuffer::open(&path)?;
        let len = fbuffer.len();
        fbuffer.prefetch(0, len);
        fbuffers.push((path, fbuffer));
    }
    for (path, fbuffer) in fbuffers {
        hasher.update(
            path.to_str()
                .ok_or(format_err!("path couldn't be read as str: {:?}", path))?
                .as_bytes(),
        );
        hasher.update(&fbuffer);
    }
    Ok(hasher.finalize())
}
