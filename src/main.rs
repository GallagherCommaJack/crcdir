use clap::{App, Arg};
use crcdir::{hash_dir, sum_dir};
use std::path::Path;
use walkdir::WalkDir;

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
