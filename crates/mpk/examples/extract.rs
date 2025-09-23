use std::{
    fs::File,
    io::Cursor,
    os::windows::fs::FileExt,
    path::{Path, PathBuf},
};

use anyhow::Context;
use binrw::BinReaderExt;
use gwynn_mpk::EntryHeader;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() -> anyhow::Result<()> {
    let mut files = vec![];
    let dir = PathBuf::from(std::env::args().nth(1).context("No dir given")?);
    for info_path in glob::glob(&dir.join("Patch*.mpkinfo").to_string_lossy())
        .unwrap()
        .flatten()
    {
        let mut f = Cursor::new(std::fs::read(&info_path)?);
        loop {
            let entry = match f.read_le::<EntryHeader>() {
                Ok(o) => o,
                Err(e) => {
                    if e.is_eof() {
                        break;
                    }

                    return Err(e.into());
                }
            };

            if entry.flags & 1 != 0 {
                // This is a directory entry, skip it.
                continue;
            }

            let data_file = info_path
                .with_extension("mpk")
                .to_string_lossy()
                .to_string();
            files.push((data_file, entry));
        }
    }

    std::fs::create_dir_all("dump")?;
    files.par_iter().for_each(|(data_file, file)| {
        let data = File::open(data_file).expect("Failed to open MPK file");

        let mut buf = vec![0; file.length as usize];
        data.seek_read(&mut buf, file.offset)
            .expect("Failed to read MPK file");

        // println!(
        //     "Extracting file '{}' with compression {:?}",
        //     &file.path,
        //     gwynn_mpk::compression::CompressionType::detect_from_slice(&buf),
        // );

        let decompressed = gwynn_mpk::compression::decompress(&mut buf)
            .expect("Failed to decompress MPK file")
            .to_vec();
        let out_path = Path::new("dump").join(&file.path);
        std::fs::create_dir_all(out_path.parent().unwrap()).expect("Failed to create directories");
        std::fs::write(&out_path, &decompressed).expect("Failed to write output file");
    });

    Ok(())
}
