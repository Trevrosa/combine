//! A Rust library that allows you to combine separate files into one, contiguously.
//!
//! This crate offers two functions for combining files: [`threaded`] and [`single`], and utilities in [`os_impl`].
//!
//! The multithreaded version should only be used when either:
//! 1. There is a large enough number of files, or;
//! 2. The total file size is large enough.
//!
//! This is because of the added cost of creating the threads and managing their inputs and outputs.
//!
//! # feature flags
//! - `tracing` -- enables logging with a tracing subscriber
//! - `cli` -- is required for compiling the example cli app

#[cfg(feature = "tracing")]
use std::time::Instant;
use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod os_impl;
use os_impl::write_all_at;

mod log;
use log::{debug, info};

/// Combine a list of files, in order, to one file using multiple threads.
///
/// If the `max_threads` is greater than `files.len()`, use that as the number of threads
///
/// `files` and `sizes` must be the same length and be in the same order.
///
/// # Examples
///
/// ```should_panic
/// # use std::path::PathBuf;
/// # use combinefiles::os_impl::file_size;
/// // these will be combined into one file contiguously
/// // so it is important that it's in order
/// let files: Vec<PathBuf> = ["a-1.zip", "a-2.zip", "a-3.zip"].iter().map(PathBuf::from).collect();
/// let sizes: Vec<u64> = files.iter().map(|f| file_size(f)).collect();
/// // combine with a max of 10 threads
/// combinefiles::threaded(files, sizes, "a.zip", 10);
/// ```
///
/// # Errors
///
/// See [`std::io::Error`].
#[allow(clippy::missing_panics_doc)]
pub fn threaded(
    files: Vec<PathBuf>,
    sizes: Vec<u64>,
    output: impl AsRef<Path>,
    max_threads: u32,
) -> io::Result<()> {
    #[cfg(feature = "tracing")]
    let start = Instant::now();

    let final_file = {
        let f = File::create_new(output)?;
        Arc::new(f)
    };

    let files_len = files.len();
    let threads = max_threads.min(files_len.try_into().expect("not that many files"));
    info!("combining {files_len} files with {max_threads} ({threads}) threads");

    // (path, offset)
    let (tx, rx) = crossbeam_channel::bounded(threads as usize);

    std::thread::spawn(move || {
        for (n, file) in files.into_iter().enumerate() {
            let offset = if n > 0 { sizes.iter().take(n).sum() } else { 0 };
            debug!("sent ({file:?}, {offset})");
            tx.send((file, offset)).expect("channel cannot be closed");
        }
        drop(tx);
    });

    let mut thread_handles = Vec::with_capacity(threads as usize);
    for i in 0..threads {
        let rx = rx.clone();
        let final_file = final_file.clone();
        thread_handles.push(std::thread::spawn(move || -> io::Result<()> {
            while let Ok((path, initial_offset)) = rx.recv() {
                info!("[thread{i}] combining {path:?} at offset {initial_offset}");

                let mut offset = initial_offset;
                let mut file = io::BufReader::new(File::open(&path).unwrap());
                loop {
                    let buf = file.fill_buf().unwrap();
                    let len = buf.len();
                    if len == 0 {
                        info!("[thread{i}] done with {path:?}");
                        break;
                    }
                    write_all_at(&final_file, buf, &mut offset)?;
                    file.consume(len);
                }

                info!("copied {} bytes from {path:?}", offset - initial_offset);
            }
            Ok(())
        }));
    }

    for handle in thread_handles {
        handle.join().unwrap()?;
    }

    info!("combined {} files in {:?}", files_len, start.elapsed());

    Ok(())
}

/// Combine a list of files, in order, to one file.
///
/// `files` and `sizes` must be the same length and be in the same order.
///
/// # Examples
///
/// ```should_panic
/// # use std::path::PathBuf;
/// # use combinefiles::os_impl::file_size;
/// // these will be combined into one file contiguously
/// // so it is important that it's in order
/// let files: Vec<PathBuf> = ["a-1.zip", "a-2.zip", "a-3.zip"].iter().map(PathBuf::from).collect();
/// let sizes: Vec<u64> = files.iter().map(|f| file_size(f)).collect();
/// // combine into "a.zip"
/// combinefiles::single(&files, &sizes, "a.zip");
/// ```
/// # Errors
///
/// See [`std::io::Error`].
pub fn single(files: &[PathBuf], sizes: &[u64], output: impl AsRef<Path>) -> io::Result<()> {
    let mut final_file = File::create_new(output)?;

    let len = files.len();
    info!("combining {len} files");

    #[cfg(feature = "tracing")]
    let start = Instant::now();

    for (n, path) in files.iter().enumerate() {
        info!("{}/{len}: combining {path:?} ({} bytes)", n + 1, sizes[n]);
        let mut file = File::open(path)?;
        io::copy(&mut file, &mut final_file)?;
    }

    info!("combined {} files in {:?}", files.len(), start.elapsed());

    Ok(())
}
