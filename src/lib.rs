use crc32fast::Hasher;
use failure::*;
use filebuffer::FileBuffer;
#[cfg(feature = "progress")]
use indicatif::*;
#[cfg(feature = "progress")]
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

#[cfg(feature = "progress")]
lazy_static! {
    pub static ref PROGRESS_BAR: ProgressBar = {
        let progbar = ProgressBar::new(0);
        progbar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} ({eta})"),
        );
        progbar
    };
}

#[cfg(feature = "progress")]
pub fn progress_init(length: u64) {
    PROGRESS_BAR.set_length(length);
    PROGRESS_BAR.set_draw_delta(length / 1000);
    PROGRESS_BAR.set_position(0);
}

pub fn hash_file<P: AsRef<Path>>(hasher: &mut Hasher, root: &Path, path: P) -> Result<(), Error> {
    match FileBuffer::open(&path) {
        Ok(fbuffer) => {
            let len = fbuffer.len();
            fbuffer.prefetch(0, len);
            hasher.update(
                path.as_ref()
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or(format_err!(
                        "path couldn't be read as str: {:?}",
                        path.as_ref()
                    ))?
                    .as_bytes(),
            );
            hasher.update(&fbuffer);
        }
        Err(e) => {
            let pathbuf = path.as_ref().to_path_buf();
            dbg!(format!(
                "failed to read filebuffer for {:?}, error was: {}",
                pathbuf, e
            ));
            let contents = std::fs::read(path)
                .map_err(move |e| format_err!("failed to open {:?}, error was: {}", pathbuf, e))?;
            hasher.update(&contents);
        }
    }
    Ok(())
}

/// Hashes an individual file using `crc32`
pub fn hash_file_oneshot<P: AsRef<Path>>(root: &Path, path: P) -> Result<u32, Error> {
    let mut hasher = Hasher::new();
    hash_file(&mut hasher, root, path)?;
    Ok(hasher.finalize())
}

/// Walks a directory, hashing the files and summing the results.
pub fn sum_dir<P: AsRef<Path> + Send + Sync>(rootdir: P) -> Result<u32, Error> {
    WalkDir::new(&rootdir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .par_bridge()
        .map(move |e| hash_file_oneshot(rootdir.as_ref(), e.path()))
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)))
}

#[cfg(feature = "progress")]
pub fn sum_dir_prog<P: AsRef<Path> + Send + Sync>(rootdir: P) -> Result<u32, Error> {
    let vec: Vec<_> = WalkDir::new(&rootdir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .collect();

    progress_init(vec.len() as u64);

    let res = vec
        .into_par_iter()
        .map(move |e| {
            let hash = hash_file_oneshot(rootdir.as_ref(), e.path());
            PROGRESS_BAR.inc(1);
            hash
        })
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)));

    PROGRESS_BAR.finish();

    res
}

/// Walks a directory, hashing the files sequentially.
pub fn hash_dir<P: AsRef<Path>>(rootdir: P) -> Result<u32, Error> {
    let mut hasher = Hasher::new();
    let vec: Vec<_> = WalkDir::new(&rootdir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .collect();

    for entry in vec.into_iter() {
        hash_file(&mut hasher, rootdir.as_ref(), entry.into_path())?;
    }

    Ok(hasher.finalize())
}

#[cfg(feature = "progress")]
pub fn hash_dir_prog<P: AsRef<Path>>(rootdir: P) -> Result<u32, Error> {
    let mut hasher = Hasher::new();
    let vec: Vec<_> = WalkDir::new(&rootdir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .collect();

    progress_init(vec.len() as u64);

    for entry in vec.into_iter() {
        let hash = hash_file(&mut hasher, rootdir.as_ref(), entry.into_path())?;
        PROGRESS_BAR.inc(1);
        hash
    }

    PROGRESS_BAR.finish();

    Ok(hasher.finalize())
}
