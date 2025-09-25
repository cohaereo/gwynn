use std::{
    fs::File,
    io::{Read, Seek},
    path::{Path, PathBuf},
    str::FromStr,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from_str(&std::env::args().nth(1).expect("No MPK file specified"))?;
    let mut data = std::fs::read(&path)?;
    let mut file = std::io::Cursor::new(&mut data);

    let mpk_info: gwynn_mpkinfo::ResourcesHeader = binrw::BinReaderExt::read_le(&mut file)?;

    let mpk_file_count = mpk_info
        .records
        .iter()
        .fold(0, |acc, e| acc.max(e.file_number() + 1));

    let base_filename = path.file_stem().unwrap().to_str().unwrap();
    let mut mpk_files = (0..mpk_file_count)
        .map(|i| {
            let filename = match i {
                0 => format!("{base_filename}.mpk"),
                _ => format!("{base_filename}{i}.mpk"),
            };

            File::open(Path::new(&path).with_file_name(filename)).expect("Failed to open MPK file")
        })
        .collect::<Vec<_>>();

    std::fs::create_dir_all("mpkinfo_dump")?;
    mpk_files
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, mpk_file)| {
            for e in mpk_info.records.iter().filter(|e| e.file_number() == i) {
                if e.is_directory() {
                    continue;
                }

                let mut data = vec![0u8; e.asset_size as usize];
                mpk_file
                    .seek(std::io::SeekFrom::Start(e.offset as u64))
                    .expect("Failed to seek");

                let hash = md5::compute(&data);
                mpk_file.read_exact(&mut data).expect("Failed to read");
                let decompressed = match gwynn_mpk::compression::decompress(&mut data) {
                    Ok(o) => o,
                    Err(err) => {
                        println!(
                            "Failed to decompress {:08X}_{:08X}.{}: {err}",
                            e.file_number(),
                            e.hash,
                            e.extension
                        );
                        continue;
                    }
                };
                let hash_decompressed = md5::compute(&decompressed);
                println!(
                    "{:08X}_{:08X}.{} - size: {} - md5: {:x} (decompressed: {:x})",
                    e.file_number(),
                    e.hash,
                    e.extension,
                    e.asset_size,
                    hash,
                    hash_decompressed
                );

                let dir = if let Some(mime) = infer::get(&decompressed) {
                    mime.extension()
                } else {
                    match decompressed.get(0..4) {
                        Some(b".MES") => {
                            continue; // Skipping MESSIAH files for now since they are large and numerous
                        }
                        Some(b"\xC1\x59\x41\x0D") => "json",
                        Some(b"N\xA1BA") | Some(b"eN\xA1B") => "nim",
                        Some(b"AKPK") => "pck",
                        Some(b"BKHD") => "bnk",
                        Some(b"CCCC") => "ory",
                        Some(b"\x0E\x00Pa") => "particlesystem",
                        Some([_, 0x0D, 0x0D, 0x0A]) => "pyc",
                        _ => match e.extension.as_str() {
                            e if e.ends_with(".0") => "unk0",
                            e if e.ends_with(".1") => {
                                continue;
                                // "tex1"
                            }
                            e if e.ends_with(".4") => {
                                continue;
                                // "tex4"
                            }
                            "nfo" => "nfo",
                            "son" => "json",
                            "onb" => "onb",
                            "csb" => "csb",
                            "ist" => "plist",
                            _ => "unknown",
                        },
                    }
                };

                std::fs::create_dir_all(format!("mpkinfo_dump/{dir}"))
                    .expect("Failed to create output directory");

                std::fs::write(
                    format!(
                        "mpkinfo_dump/{dir}/{:08X}_{:08X}.{}",
                        e.file_number(),
                        e.hash,
                        e.extension
                    ),
                    decompressed,
                )
                .expect("Failed to write file");
            }
        });
    // for e in mpk_info.records {
    //     if e.is_directory() {
    //         continue;
    //     }

    //     let mut output = vec![0u8; e.asset_size as usize];
    //     let mpk_file = &mut mpk_files[e.file_number() as usize];
    //     mpk_file.seek(std::io::SeekFrom::Start(e.offset as u64))?;
    //     mpk_file.read_exact(&mut output)?;
    //     std::fs::write(
    //         format!(
    //             "mpkinfo_dump/{:08X}_{:08X}.{}",
    //             e.file_number(),
    //             e.hash,
    //             e.extension
    //         ),
    //         output,
    //     )?;
    // }

    Ok(())
}
