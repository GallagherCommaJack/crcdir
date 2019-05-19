use clap::{App, Arg};
use crc32fast::Hasher;
use failure::*;
use rayon::prelude::*;
use std::{fs::File, io::Read, path::Path};
use walkdir::WalkDir;

fn hash_file(path: &Path) -> Result<u32, Error> {
    let mut hasher = Hasher::new();
    hasher.update(
        path.to_str()
            .ok_or(format_err!("path couldn't be read as str: {:?}", path))?
            .as_bytes(),
    );
    let mut buf = vec![];
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    hasher.update(&buf);
    Ok(hasher.finalize())
}

fn sum_dir(path: WalkDir) -> Result<u32, Error> {
    path.into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .par_bridge()
        .map(|e| hash_file(e.path()))
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)))
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

fn main() {
    let args = App::new("crcdir")
        .version("0.1")
        .arg(Arg::with_name("DIR").help("directory to checksum").index(1))
        .arg(
            Arg::with_name("sum")
                .short("-s")
                .long("--sum")
                .help("checksum directory instead of serially hashing"),
        )
        .get_matches();

    let path = args.value_of("INPUT").unwrap_or_else(|| ".");
    let walker = WalkDir::new(Path::new(path));
    let val = if args.is_present("sum") {
        sum_dir(walker).unwrap_or_else(|e| panic!("{:?}", e))
    } else {
        hash_dir(walker).expect("couldn't checksum - found no files")
    };
    println!("{:08x}", val);
}
