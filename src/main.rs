use clap::{App, Arg};
use crc32fast::Hasher;
use failure::*;
use filebuffer::FileBuffer;
use rayon::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

fn hash_file(path: &Path) -> Result<u32, Error> {
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

fn sum_dir(path: WalkDir) -> Result<u32, Error> {
    path.into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .par_bridge()
        .map(|e| hash_file(e.path()))
        .reduce(|| Ok(0u32), |acc, i| Ok(acc?.wrapping_add(i?)))
}

fn hash_dir(path: WalkDir) -> Result<u32, Error> {
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

fn main() {
    let args = App::new("crcdir")
        .version("0.1")
        .arg(Arg::with_name("DIR").help("directory to checksum").index(1))
        .arg(
            Arg::with_name("serial")
                .short("-s")
                .long("--serial")
                .help("serially hash directory instead of checksumming"),
        )
        .arg(
            Arg::with_name("jobs")
                .short("-j")
                .long("--jobs")
                .takes_value(true)
                .help("number of jobs to use when checksumming"),
        )
        .get_matches();

    let path = args.value_of("INPUT").unwrap_or_else(|| ".");
    let walker = WalkDir::new(Path::new(path));
    let val = if args.is_present("serial") {
        hash_dir(walker).expect("couldn't checksum - found no files")
    } else {
        let jobs: usize = if let Some(n) = args.value_of("jobs") {
            n.parse().unwrap_or_else(|e| panic!("{:?}", e))
        } else {
            num_cpus::get()
        };
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap()
            .install(|| sum_dir(walker).unwrap_or_else(|e| panic!("{:?}", e)))
    };
    println!("{:08x}", val);
}
