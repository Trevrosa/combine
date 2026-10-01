//! Operating-system dependent functions.

use std::{fs::File, io, path::Path};

/// Get the size of a file from its metadata.
///
/// # Panics
/// Fails if the metadata could not be read.
#[cfg(windows)]
#[must_use]
pub fn file_size(p: &Path) -> u64 {
    use std::os::windows::fs::MetadataExt;
    p.metadata().map(|m| m.file_size()).unwrap()
}

/// Get the size of a file from its metadata.
///
/// # Panics
/// Fails if the metadata could not be read.
#[cfg(unix)]
#[must_use]
pub fn file_size(p: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    p.metadata().map(|m| m.size()).unwrap()
}

/// Attempts to write an entire buffer starting from a given offset.
///
/// The offset is relative to the start of the file and thus independent from the current cursor.
///
/// The current file cursor is not affected by this function.
///
/// # Errors
/// This function will return the first error of non-[`io::ErrorKind::Interrupted`] kind that a write returns.
#[cfg(unix)]
pub fn write_all_at(writer: &File, buf: &[u8], offset: &mut u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;
    let write = writer.write_all_at(buf, *offset);
    *offset += buf.len() as u64;
    write
}

/// Attempts to write an entire buffer starting from a given offset.
///
/// The offset is relative to the start of the file and thus independent from the current cursor.
///
/// The current file cursor is not affected by this function.
///
/// # Errors
/// This function will return the first error of non-[`io::ErrorKind::Interrupted`] kind that a write returns.
#[cfg(windows)]
pub fn write_all_at(writer: &File, mut buf: &[u8], offset: &mut u64) -> io::Result<()> {
    // Copies implementation from unix::fs::FileExt
    use io::ErrorKind::{Interrupted, WriteZero};
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        match writer.seek_write(buf, *offset) {
            Ok(0) => return Err(io::Error::new(WriteZero, "failed to write full buffer")),
            Ok(n) => {
                buf = &buf[n..];
                *offset += n as u64;
            }
            Err(ref e) if matches!(e.kind(), Interrupted) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
