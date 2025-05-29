use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

use anyhow::Context;
use binrw::BinReaderExt;
use gwynn_texture::{converter::TextureConverter, TextureHeader};
#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let texture_converter = TextureConverter::new()?;

    for path in glob::glob("textures_in/**/*.4")?.flatten() {
        let mut f = File::open(&path).unwrap();
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();

        let mut c = Cursor::new(&buf);

        let texture_header: TextureHeader = c.read_le()?;
        let mip = texture_header.mips.last().unwrap();
        info!(
            "{:?} {}x{} / {}x{} {}",
            texture_header.format,
            texture_header.width,
            texture_header.height,
            mip.width,
            mip.height,
            path.display(),
        );

        println!("- 0x{:X} => 0x{:X}", mip.data_offset.pos, mip.data_end.pos);

        let texture_data = gwynn_mpk::compression::decompress(
            // &mut buf[mip.data_offset.pos as usize..mip.data_end.pos as usize],
            &mut buf[mip.data_offset.pos as usize..],
        )?;

        let out_file = Path::new("textures_out").join(path.file_name().unwrap());

        let image_data = match texture_converter.convert(&texture_data, &texture_header) {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to convert texture: {e}");
                continue;
            }
        };

        if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            mip.width as u32,
            mip.height as u32,
            &image_data[..],
        ) {
            output_image.save(out_file.with_extension("png"))?;
        }
    }

    Ok(())
}
