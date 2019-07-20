use clap::{App, Arg};
use crcdir::*;

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

    let path = args.value_of("DIR").unwrap_or_else(|| ".");
    let val = if args.is_present("serial") {
        #[cfg(feature = "progress")]
        {
            hash_dir_prog(path).expect("couldn't checksum - found no files")
        }
        #[cfg(not(feature = "progress"))]
        {
            hash_dir(path).expect("couldn't checksum - found no files")
        }
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
            .install(|| {
                #[cfg(feature = "progress")]
                let res = sum_dir_prog(path);
                #[cfg(not(feature = "progress"))]
                let res = sum_dir(path);
                res.unwrap_or_else(|e| panic!("{:?}", e))
            })
    };
    println!("{:08x}", val);
}
