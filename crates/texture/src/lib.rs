use std::{borrow::Cow, io::Cursor};

use anyhow::Context;
use binrw::BinReaderExt;
use log::debug;
use wgpu::util::DeviceExt;

use crate::structs::TextureHeader;

pub mod converter;
pub mod format;
pub mod structs;

/// Represents a Messiah texture loaded onto the GPU
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub header: TextureHeader,
}

impl Texture {
    /// Load a texture from raw Messiah texture file data
    pub fn load(device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) -> anyhow::Result<Self> {
        let mut c = Cursor::new(&data);
        let _unk0: u32 = c.read_le()?;

        let header: TextureHeader = c.read_le()?;
        let mip = header.mips.last().context("Texture has no mips?")?;

        let mut data = data[mip.data_offset.pos as usize..].to_vec();
        let texture_data = gwynn_mpk::compression::decompress(&mut data)?;
        let texture_data = if header.format.is_astc()
            && !device
                .features()
                .contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC)
        {
            debug!("Using ASTC software decoder");
            let mut image = vec![0u8; header.width as usize * header.height as usize * 4];
            astc_decode::astc_decode(
                Cursor::new(&data),
                header.width as _,
                header.height as _,
                header.format.astc_footprint().unwrap(),
                |x, y, pixel| {
                    let offset = (y * header.width as u32 + x) as usize * 4;
                    image[offset..offset + 4].copy_from_slice(&pixel);
                },
            )?;

            Cow::Owned(image)
        } else {
            texture_data
        };

        let format = header.format.to_wgpu().with_context(|| {
            format!(
                "No suitable WGPU format found for format {:?}",
                header.format
            )
        })?;

        let block_size = format.block_dimensions();
        let texture_size_aligned = wgpu::Extent3d {
            width: (header.width as f32 / block_size.0 as f32).ceil() as u32 * block_size.0,
            height: (header.height as f32 / block_size.1 as f32).ceil() as u32 * block_size.1,
            ..Default::default()
        };

        let block_count: u32 = (texture_size_aligned.width / block_size.0)
            * (texture_size_aligned.height / block_size.1);
        let block_size = format.block_copy_size(None).unwrap_or(1);

        if texture_data.len() < (block_count * block_size) as usize {
            anyhow::bail!("Insufficient data for texture conversion");
        }

        if format.has_depth_aspect() {
            anyhow::bail!("Depth textures are not supported");
        }

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: texture_size_aligned,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[format],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            texture_data.as_ref(),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            header,
        })
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.header.width as f32 / self.header.height as f32
    }
}
