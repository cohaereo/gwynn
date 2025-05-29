use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use binrw::{binread, BinReaderExt};

#[binread]
struct ShaderHash {
    pub hash: [u8; 16],
    pub size: u32,
}

#[binread]
struct ShaderCacheHeader {
    #[br(temp)]
    count: u32,
    #[br(count = count)]
    pub blobs: Vec<ShaderHash>,
}

fn main() -> anyhow::Result<()> {
    let filepath = PathBuf::from(std::env::args().nth(1).unwrap());
    let mut file = BufReader::new(File::open(&filepath)?);
    let filename = filepath.file_name().unwrap().to_string_lossy().to_string();
    println!("{filename}");

    let is_vlk = filepath
        .components()
        .find(|c| c.as_os_str().to_string_lossy() == "vlk")
        .is_some();

    let header: ShaderCacheHeader = file.read_le()?;

    let out_dir = PathBuf::from("shaders").join(&filename);
    for b in header.blobs.iter() {
        let mut data = vec![0u8; b.size as usize];
        file.read_exact(&mut data)?;

        let decompressed = gwynn_mpk::compression::decompress(&mut data)?;
        std::fs::create_dir_all(&out_dir)?;

        if is_vlk {
            std::fs::write(
                out_dir.join(format!("{}.spv", hex::encode(&b.hash))),
                &decompressed,
            )?;
        } else {
            std::fs::write(
                out_dir.join(format!("{}.glsl", hex::encode(&b.hash))),
                &decompressed[0..decompressed
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(decompressed.len())],
            )?;
        }
    }

    Ok(())
}
