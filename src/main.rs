use std::{
    fs::File,
    io::{BufReader, Cursor, Seek, Write},
    os::windows::fs::FileExt,
    path::{Path, PathBuf},
};

use anyhow::Context;
use binrw::{meta::ReadMagic, BinReaderExt};
use gwynn_model::header::{MessiahFileType, MessiahHeader, ModelHeader};
use gwynn_mpk::EntryHeader;
use gwynn_texture::{converter::TextureConverter, TextureHeader};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let texture_converter = TextureConverter::new()?;

    let dir = PathBuf::from(std::env::args().nth(1).context("No dir given")?);
    glob::glob(&dir.join("Patch*.mpkinfo").to_string_lossy())?
        .flatten()
        .par_bridge()
        .for_each(|info_path| {
            let result: anyhow::Result<()> = (|| {
                println!("Reading {}", info_path.display());
                let data_path = info_path.with_extension("mpk");
                let data = File::open(data_path)?;
                let mut f = BufReader::new(File::open(info_path)?);
                let mut entries = vec![];
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

                    entries.push(entry);
                }

                // entries.sort_by_key(|e| e.length);
                // entries.reverse();

                // entries.par_iter().for_each(|entry| {
                for entry in &entries {
                    let result: anyhow::Result<()> = (|| {
                        if entry.is_directory() {
                            return Ok(());
                        }

                        // Skip textures
                        if entry.path.contains(".") {
                            return Ok(());
                        }

                        // if !entry.path.contains("01afcd89-2d91-5e3e-aefc-e205bcc5ac53") {
                        //     return Ok(());
                        // }

                        let mut buf = vec![0; entry.length as usize];
                        data.seek_read(&mut buf, entry.offset)?;

                        // info!(
                        //     "Compression {:?} - {}",
                        //     gwynn_mpk::compression::CompressionType::guess_from_slice(&buf),
                        //     &entry.path
                        // );
                        let decompressed = gwynn_mpk::compression::decompress(&mut buf)?.to_vec();
                        // let out_dir = PathBuf::from("out");
                        // let out_file = out_dir.join(PathBuf::from(&entry.path).file_name().unwrap());
                        // std::fs::create_dir_all(out_file.parent().unwrap())?;
                        // let mut f = File::create(&out_file)
                        //     .with_context(|| format!("Failed to create file {}", out_file.display()))?;
                        // f.write_all(&decompressed)?;
                        let out_dir = PathBuf::from("C:/Users/Luca/Downloads/models");
                        let out_file =
                            out_dir.join(PathBuf::from(&entry.path).file_name().unwrap());
                        // std::fs::create_dir_all(out_file.parent().unwrap())?;

                        // Not a messiah file, skip it
                        if !decompressed.starts_with(&MessiahHeader::MAGIC) {
                            return Ok(());
                        }
                        let mut c = Cursor::new(&decompressed);

                        let messiah = match c.read_le::<MessiahHeader>() {
                            Ok(m) => m,
                            Err(e) => {
                                File::create(format!(
                                    "unknown messiah type ({}).bin",
                                    Path::new(&entry.path)
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy()
                                ))?
                                .write_all(&decompressed)?;
                                panic!("Failed to read messiah header for {}: {e}", &entry.path);
                                return Ok(());
                            }
                        };
                        return Ok(());

                        if messiah.file_type != MessiahFileType::Model {
                            if messiah.file_type == MessiahFileType::Material {
                                // File::create(format!(
                                //     "{}.mat",
                                //     Path::new(&entry.path)
                                //         .file_name()
                                //         .unwrap()
                                //         .to_string_lossy()
                                // ))?
                                // .write_all(&decompressed)?;
                                info!("Skipping material {}", entry.path);
                            }
                            return Ok(());
                        }
                        let model: ModelHeader = c.read_le()?;
                        // println!("{model:#X?}");

                        // for buffer in &model.buffer_layouts {
                        //     println!("{:#?}", buffer.to_buffer_layout());
                        // }

                        if !model.buffer_layouts[0].string.contains("P3F") {
                            warn!("Missing position buffer for model {}", entry.path);
                            return Ok(());
                        }

                        let mut f =
                            File::create(&out_file.with_extension("obj")).with_context(|| {
                                format!("Failed to create file {}", out_file.display())
                            })?;

                        writeln!(&mut f, "# {}", entry.path);
                        writeln!(
                            &mut f,
                            "o {}",
                            out_file.file_name().unwrap().to_string_lossy()
                        );

                        let indices = model
                            .read_indices_u32(&mut c)
                            .context("Failed to read indices")?;

                        let vertex_data_offset = model.vertex_buffer_offset();
                        let buffer_layout = model.buffer_layouts[0].to_buffer_layout()?.unwrap();
                        for vertex in 0..model.vertex_count as u64 {
                            c.seek(std::io::SeekFrom::Start(
                                vertex_data_offset + vertex * buffer_layout.stride() as u64,
                            ))?;

                            let vertex: [f32; 3] = c.read_le()?;
                            writeln!(&mut f, "v {} {} {}", vertex[0], vertex[1], vertex[2])?;
                        }

                        for face in indices.chunks_exact(3) {
                            writeln!(&mut f, "f {} {} {}", face[0] + 1, face[1] + 1, face[2] + 1)?;
                        }

                        // let texture_header: TextureHeader = c.read_le()?;
                        // let mip = texture_header.mips.last().unwrap();
                        // info!(
                        //     "{:?} {}x{} / {}x{} {}",
                        //     texture_header.format,
                        //     texture_header.width,
                        //     texture_header.height,
                        //     mip.width,
                        //     mip.height,
                        //     entry.path
                        // );

                        // let texture_data = gwynn_mpk::compression::decompress(
                        //     &mut decompressed[mip.data_offset.pos as usize..mip.data_end.pos as usize],
                        // )
                        // .context("Failed to decompress texture data")?;

                        // // let mut f = File::create(&out_file.with_extension("data"))
                        // //     .with_context(|| format!("Failed to create file {}", out_file.display()))?;
                        // // f.write_all(&texture_data)?;

                        // let image_data = texture_converter.convert(&texture_data, &texture_header)?;

                        // if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                        //     mip.width as u32,
                        //     mip.height as u32,
                        //     &image_data[..],
                        // ) {
                        //     output_image.save(out_file.with_extension("png"))?;
                        // }
                        Ok(())
                    })();

                    if let Err(e) = result {
                        error!("Failed to read file: {}", e);
                    }
                }

                Ok(())
                // });
            })();

            if let Err(e) = result {
                error!("Failed to read file: {}", e);
            }
        });

    Ok(())
}
