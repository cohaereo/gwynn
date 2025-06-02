pub trait ReaderExt {
    fn seek_read(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
}

impl<R> ReaderExt for R
where
    R: std::io::Read + std::io::Seek,
{
    fn seek_read(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.seek(std::io::SeekFrom::Start(offset))?;
        self.read(buf)
    }
}

pub struct PatchManager {}
