use clap::{App, Arg};
use crc32fast::Hasher;
use std::{fs::File, io::Read, path::Path};
use walkdir::WalkDir;

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
    let matches = App::new("crcdir")
        .version("0.1")
        .arg(Arg::with_name("DIR").help("directory to checksum").index(1))
        .get_matches();

    let path = matches.value_of("INPUT").unwrap_or_else(|| ".");
    let walker = WalkDir::new(Path::new(path));
    println!(
        "{:x}",
        hash_dir(walker).expect("couldn't checksum - found no files")
    );
}
