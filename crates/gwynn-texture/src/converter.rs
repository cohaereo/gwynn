use std::io::Cursor;

use anyhow::Context;
use futures::executor::block_on;
use tracing::{debug, warn};
use wgpu::{
    util::DeviceExt, ComputePipelineDescriptor, DeviceDescriptor, Extent3d, Features, Limits,
    PipelineCompilationOptions, RequestAdapterOptions, ShaderModuleDescriptor, TextureDescriptor,
    TextureUsages,
};

use crate::TextureHeader;

pub struct TextureConverter {
    _instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pipeline: wgpu::ComputePipeline,
}

impl TextureConverter {
    pub fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let adapter =
            futures::executor::block_on(instance.request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }))
            .context("Couldn't get wgpu adapter")?;

        let mut features = Features::TEXTURE_COMPRESSION_BC;
        if adapter
            .features()
            .contains(Features::TEXTURE_COMPRESSION_ASTC)
        {
            features |= Features::TEXTURE_COMPRESSION_ASTC;
        } else {
            warn!("GPU does not support ASTC textures. Decoding will be done on the CPU");
        };

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Texture Converter Device"),
                required_features: features,
                required_limits: Limits {
                    max_texture_dimension_2d: 4096,
                    ..Limits::downlevel_defaults()
                },
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))?;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Texture converter shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./convert.wgsl").into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Texture converter pipeline"),
            layout: None,
            module: &shader,
            entry_point: "convert_main",
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            _instance: instance,
            device,
            queue,
            pipeline,
        })
    }

    /// Converts the given texture data to RGBA8888
    pub fn convert(&self, data: &[u8], header: &TextureHeader) -> anyhow::Result<Vec<u8>> {
        debug!("Converting texture");

        if header.format.is_hdr() {
            anyhow::bail!("HDR textures are not supported");
        }

        if header.format.is_astc()
            && !self
                .device
                .features()
                .contains(Features::TEXTURE_COMPRESSION_ASTC)
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

            return Ok(image);
        };

        let format = header.format.to_wgpu().with_context(|| {
            format!(
                "No suitable WGPU format found for format {:?}",
                header.format
            )
        })?;

        let block_size = format.block_dimensions();
        let texture_size = Extent3d {
            width: header.width as _,
            height: header.height as _,
            ..Default::default()
        };
        let texture_size_aligned = Extent3d {
            width: (header.width as f32 / block_size.0 as f32).ceil() as u32 * block_size.0,
            height: (header.height as f32 / block_size.1 as f32).ceil() as u32 * block_size.1,
            ..Default::default()
        };

        let block_count: u32 = (texture_size_aligned.width / block_size.0)
            * (texture_size_aligned.height / block_size.1);
        let block_size = format.block_copy_size(None).unwrap_or(1);

        if data.len() < (block_count * block_size) as usize {
            anyhow::bail!("Insufficient data for texture conversion");
        }

        if format.has_depth_aspect() {
            anyhow::bail!("Depth textures are not supported");
        }

        let input_texture = self.device.create_texture_with_data(
            &self.queue,
            &TextureDescriptor {
                label: None,
                size: texture_size_aligned,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[format],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );

        // Create an output texture
        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let (dispatch_with, dispatch_height) =
                compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Copy pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &texture_bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
        }

        debug!("Downloading converted texture data");

        // Get the result.

        let padded_bytes_per_row = padded_bytes_per_row(texture_size.width);
        let unpadded_bytes_per_row = texture_size.width as usize * 4;

        let output_buffer_size = padded_bytes_per_row as u64
            * texture_size.height as u64
            * std::mem::size_of::<u8>() as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(texture_size.height as u32),
                },
            },
            texture_size,
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

        self.device.poll(wgpu::Maintain::Wait);

        let padded_data = buffer_slice.get_mapped_range();

        let mut pixels: Vec<u8> = vec![0; unpadded_bytes_per_row * texture_size.height as usize];
        for (padded, pixels) in padded_data
            .chunks_exact(padded_bytes_per_row)
            .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
        {
            pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
        }

        Ok(pixels)
    }
}

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

/// Compute the next multiple of 256 for texture retrieval padding.
fn padded_bytes_per_row(width: u32) -> usize {
    let bytes_per_row = width as usize * 4;
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}
