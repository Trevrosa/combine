use std::{fs::File, io, path::Path};

#[cfg(windows)]
pub fn file_size(p: &Path) -> u64 {
    use std::os::windows::fs::MetadataExt;
    p.metadata().map(|m| m.file_size()).unwrap()
}

#[cfg(unix)]
pub fn file_size(p: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    p.metadata().map(|m| m.size()).unwrap()
}

#[cfg(unix)]
pub fn write_all_at(writer: &File, buf: &[u8], offset: &mut u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;
    *offset += buf.len() as u64;
    writer.write_all_at(buf, *offset)
}

#[cfg(windows)]
pub fn write_all_at(writer: &File, buf: &[u8], offset: &mut u64) -> io::Result<()> {
    use io::ErrorKind::{Interrupted, WriteZero};
    while !buf.is_empty() {
        match final_file.seek_write(buf, offset) {
            Ok(0) => return io::Error::new(WriteZero, "failed to write full buffer"),
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
