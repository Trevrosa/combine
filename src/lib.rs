#[cfg(feature = "tracing")]
use std::time::Instant;
use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::Arc,
};

mod platforms;
pub use platforms::{file_size, write_all_at};

mod log;
#[cfg(feature = "tracing")]
use tracing::{debug, error, info, warn};

/// Combine `files` into one file named `output_name` with a max number of threads `max_threads`.
///
/// `files` will be combined in order
pub fn threaded(
    files: Vec<PathBuf>,
    sizes: Vec<u64>,
    output: &Path,
    max_threads: u16,
) -> io::Result<()> {
    #[cfg(feature = "tracing")]
    let start = Instant::now();

    let final_file = {
        let f = File::create_new(output)?;
        Arc::new(f)
    };

    let files_len = files.len();
    let threads =
        max_threads.min(u16::try_from(files_len).expect("should not be that many chunks"));
    info!("combining {files_len} files with {max_threads} ({threads}) threads");

    // (path, offset)
    let (tx, rx) = crossbeam_channel::bounded(threads as usize);

    std::thread::spawn(move || {
        for (n, file) in files.into_iter().enumerate() {
            let offset = if n > 0 { sizes.iter().take(n).sum() } else { 0 };
            tx.send((file, offset)).expect("channel cannot be closed");
        }
        drop(tx);
    });

    let mut thread_handles = Vec::with_capacity(threads as usize);
    for i in 0..threads {
        let rx = rx.clone();
        let final_file = final_file.clone();
        thread_handles.push(std::thread::spawn(move || -> io::Result<()> {
            let Ok((path, initial_offset)) = rx.recv() else {
                return Ok(());
            };
            println!("[thread{i}] combining {path:?} at offset {initial_offset}");

            let mut offset = initial_offset;
            let mut file = io::BufReader::new(File::open(&path).unwrap());
            loop {
                let buf = file.fill_buf().unwrap();
                let len = buf.len();
                if len == 0 {
                    println!("[thread{i}] done with {path:?}");
                    break;
                }
                write_all_at(&final_file, buf, &mut offset)?;
                file.consume(len);
            }

            let total_written = offset - initial_offset;
            info!("copied {total_written} bytes from {path:?}");
            Ok(())
        }));
    }

    for handle in thread_handles {
        handle.join().unwrap()?;
    }

    info!("combined {} files in {:?}", files_len, start.elapsed());

    Ok(())
}

pub fn single(files: &[PathBuf], output: &Path) -> io::Result<()> {
    let mut final_file = File::create_new(output)?;

    let len = files.len();
    info!("combining {len} files");

    #[cfg(feature = "tracing")]
    let start = Instant::now();

    for (n, file) in files.iter().enumerate() {
        let size = file_size(&file);
        let mut file = File::open(file)?;
        io::copy(&mut file, &mut final_file)?;
        info!("{}/{len}: combining {file:?} ({size} bytes)", n + 1);
    }

    info!("combined {} files in {:?}", files.len(), start.elapsed());

    Ok(())
}
