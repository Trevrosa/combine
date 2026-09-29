use clap::Parser;
use combinefiles::os_impl::file_size;
use std::{path::PathBuf, process::exit, thread::available_parallelism};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    output: PathBuf,

    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Force multithreaded mode
    #[arg(short = 'F', long)]
    force_threaded: bool,

    /// The max number of threads to use, if multithreaded
    #[arg(short, long)]
    threads: Option<u16>,
}

/// bytes
const SMALL_FILE: u64 = 1024 * 1024 * 1024; // 1 GiB

fn main() {
    let Args {
        output,
        files,
        force_threaded,
        threads,
    } = Args::parse();

    let threads = threads.unwrap_or_else(|| available_parallelism().unwrap().get() as u16);

    for file in &files {
        if !file.try_exists().is_ok_and(|exists| exists) {
            eprintln!("{file:?} does not exist!");
            exit(1);
        }
    }

    let sizes: Vec<u64> = files.iter().map(|f| file_size(f)).collect();

    if force_threaded || sizes.iter().sum::<u64>() > SMALL_FILE {
        combinefiles::threaded(files, sizes, &output, threads as u32).unwrap();
    } else {
        combinefiles::single(&files, &sizes, &output).unwrap();
    }
}
