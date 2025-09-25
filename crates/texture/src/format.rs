use binrw::binread;

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum PixelFormat {
    Unknown = 0,
    A32R32G32B32F = 1,
    A16B16G16R16F = 2,
    R8G8B8A8 = 3,

    B5G6R5 = 6,
    A8L8 = 7,
    G16R16 = 8,
    G16R16F = 9,
    G32R32F = 10,
    R16F = 11,
    L8 = 12,
    L16 = 13,
    A8 = 14,
    FloatRGB = 15,
    FloatRGBA = 255, // ??
    D24 = 16,
    D32 = 17,
    Bc1 = 18,
    Bc2 = 19,
    Bc3 = 20,
    Bc4 = 21,
    Bc5 = 22,
    Bc6S = 23,
    Bc6U = 24,
    Bc7 = 25,
    Pvrtc2Rgb = 27,
    Pvrtc2Rgba = 28,
    Pvrtc4Rgb = 29,
    Pvrtc4Rgba = 30,
    Etc1 = 31,
    Etc2Rgb = 32,
    Etc2Rgba = 33,
    AtcRgbaE = 34,
    AtcRgbaI = 35,
    Astc4x4Ldr = 36,
    Astc5x4Ldr = 37,
    Astc5x5Ldr = 38,
    Astc6x5Ldr = 39,
    Astc6x6Ldr = 40,
    Astc8x5Ldr = 41,
    Astc8x6Ldr = 42,
    Astc8x8Ldr = 43,
    Astc10x5Ldr = 44,
    Astc10x6Ldr = 45,
    Astc10x8Ldr = 46,
    Astc10x10Ldr = 47,
    Astc12x10Ldr = 48,
    Astc12x12Ldr = 49,
    Shadowdepth = 50,
    Shadowdepth32 = 51,
    R10g10b10a2 = 52,
    R32u = 53,
    R11g11b10 = 54,
    Astc4x4Hdr = 55,
    Astc5x4Hdr = 56,
    Astc5x5Hdr = 57,
    Astc6x5Hdr = 58,
    Astc6x6Hdr = 59,
    Astc8x5Hdr = 60,
    Astc8x6Hdr = 61,
    Astc8x8Hdr = 62,
    Astc10x5Hdr = 63,
    Astc10x6Hdr = 64,
    Astc10x8Hdr = 65,
    Astc10x10Hdr = 66,
    Astc12x10Hdr = 67,
    Astc12x12Hdr = 68,
    R32g32b32a32ui = 69,
}

impl PixelFormat {
    pub fn to_wgpu(&self) -> Option<wgpu::TextureFormat> {
        Some(match self {
            PixelFormat::Unknown => return None,
            PixelFormat::A32R32G32B32F => return None,
            // PixelFormat::A16B16G16R16F => return None,
            PixelFormat::R8G8B8A8 => wgpu::TextureFormat::Rgba8Unorm,
            PixelFormat::B5G6R5 => return None,
            PixelFormat::A8L8 => return None,
            PixelFormat::G16R16 => return None,
            PixelFormat::G16R16F => return None,
            PixelFormat::G32R32F => return None,
            // PixelFormat::R32F => wgpu::TextureFormat::R32Float,
            PixelFormat::R16F => wgpu::TextureFormat::R16Float,
            PixelFormat::L8 => wgpu::TextureFormat::R8Unorm,
            PixelFormat::L16 => wgpu::TextureFormat::R16Unorm,
            PixelFormat::A8 => wgpu::TextureFormat::R8Unorm,
            PixelFormat::FloatRGB => return None,
            PixelFormat::FloatRGBA => wgpu::TextureFormat::Rgba32Float,
            PixelFormat::D24 => wgpu::TextureFormat::Depth24Plus,
            PixelFormat::D32 => wgpu::TextureFormat::Depth32Float,
            PixelFormat::Bc1 => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            PixelFormat::Bc2 => wgpu::TextureFormat::Bc2RgbaUnormSrgb,
            PixelFormat::Bc3 => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
            PixelFormat::Bc4 => wgpu::TextureFormat::Bc4RUnorm,
            PixelFormat::Bc5 => wgpu::TextureFormat::Bc5RgUnorm,
            PixelFormat::Bc6S => wgpu::TextureFormat::Bc6hRgbFloat,
            PixelFormat::Bc6U => wgpu::TextureFormat::Bc6hRgbUfloat,
            PixelFormat::Bc7 => wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            PixelFormat::Pvrtc2Rgb => return None,
            PixelFormat::Pvrtc2Rgba => return None,
            PixelFormat::Pvrtc4Rgb => return None,
            PixelFormat::Etc1 => return None,
            PixelFormat::Etc2Rgb => wgpu::TextureFormat::Etc2Rgb8UnormSrgb,
            PixelFormat::Etc2Rgba => wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb,
            // PixelFormat::AtcRgb => return None,
            PixelFormat::AtcRgbaE => return None,
            PixelFormat::AtcRgbaI => return None,
            PixelFormat::Astc4x4Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B4x4,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc5x4Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B5x4,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc5x5Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B5x5,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc6x5Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B6x5,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc6x6Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B6x6,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc8x5Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x5,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc8x6Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x6,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc8x8Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x8,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc10x5Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x5,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc10x6Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x6,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc10x8Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x8,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc10x10Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x10,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc12x10Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B12x10,
                channel: wgpu::AstcChannel::Unorm,
            },
            PixelFormat::Astc12x12Ldr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B12x12,
                channel: wgpu::AstcChannel::Unorm,
            },
            // PixelFormat::DepthStencil => wgpu::TextureFormat::Depth24PlusStencil8,
            PixelFormat::Shadowdepth => wgpu::TextureFormat::Depth24Plus, // ??
            PixelFormat::Shadowdepth32 => wgpu::TextureFormat::Depth32Float,
            PixelFormat::R10g10b10a2 => wgpu::TextureFormat::Rgb10a2Unorm,
            PixelFormat::R32u => wgpu::TextureFormat::R32Uint,
            // PixelFormat::R11g11b10f => wgpu::TextureFormat::Rg11b10Ufloat,
            PixelFormat::Astc4x4Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B4x4,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc5x4Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B5x4,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc5x5Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B5x5,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc6x5Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B6x5,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc6x6Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B6x6,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc8x5Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x5,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc8x6Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x6,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc8x8Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B8x8,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc10x5Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x5,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc10x6Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x6,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc10x8Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x8,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc10x10Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B10x10,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc12x10Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B12x10,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::Astc12x12Hdr => wgpu::TextureFormat::Astc {
                block: wgpu::AstcBlock::B12x12,
                channel: wgpu::AstcChannel::Hdr,
            },
            PixelFormat::A16B16G16R16F => return None,
            PixelFormat::Pvrtc4Rgba => return None,
            PixelFormat::R11g11b10 => wgpu::TextureFormat::Rg11b10Ufloat,
            PixelFormat::R32g32b32a32ui => wgpu::TextureFormat::Rgba32Uint,
            // PixelFormat::A32r32g32b32ui => return None,
        })
    }

    pub fn astc_footprint(&self) -> Option<astc_decode::Footprint> {
        Some(match self {
            PixelFormat::Astc4x4Ldr => astc_decode::Footprint::ASTC_4X4,
            PixelFormat::Astc5x4Ldr => astc_decode::Footprint::ASTC_5X4,
            PixelFormat::Astc5x5Ldr => astc_decode::Footprint::ASTC_5X5,
            PixelFormat::Astc6x5Ldr => astc_decode::Footprint::ASTC_6X5,
            PixelFormat::Astc6x6Ldr => astc_decode::Footprint::ASTC_6X6,
            PixelFormat::Astc8x5Ldr => astc_decode::Footprint::ASTC_8X5,
            PixelFormat::Astc8x6Ldr => astc_decode::Footprint::ASTC_8X6,
            PixelFormat::Astc8x8Ldr => astc_decode::Footprint::ASTC_8X8,
            PixelFormat::Astc10x5Ldr => astc_decode::Footprint::ASTC_10X5,
            PixelFormat::Astc10x6Ldr => astc_decode::Footprint::ASTC_10X6,
            PixelFormat::Astc10x8Ldr => astc_decode::Footprint::ASTC_10X8,
            PixelFormat::Astc10x10Ldr => astc_decode::Footprint::ASTC_10X10,
            PixelFormat::Astc12x10Ldr => astc_decode::Footprint::ASTC_12X10,
            PixelFormat::Astc12x12Ldr => astc_decode::Footprint::ASTC_12X12,
            _ => return None,
        })
    }

    pub fn is_astc(&self) -> bool {
        matches!(
            self,
            Self::Astc4x4Ldr
                | Self::Astc5x4Ldr
                | Self::Astc5x5Ldr
                | Self::Astc6x5Ldr
                | Self::Astc6x6Ldr
                | Self::Astc8x5Ldr
                | Self::Astc8x6Ldr
                | Self::Astc8x8Ldr
                | Self::Astc10x5Ldr
                | Self::Astc10x6Ldr
                | Self::Astc10x8Ldr
                | Self::Astc10x10Ldr
                | Self::Astc12x10Ldr
                | Self::Astc12x12Ldr
                | Self::Astc4x4Hdr
                | Self::Astc5x4Hdr
                | Self::Astc5x5Hdr
                | Self::Astc6x5Hdr
                | Self::Astc6x6Hdr
                | Self::Astc8x5Hdr
                | Self::Astc8x6Hdr
                | Self::Astc8x8Hdr
                | Self::Astc10x5Hdr
                | Self::Astc10x6Hdr
                | Self::Astc10x8Hdr
                | Self::Astc10x10Hdr
                | Self::Astc12x10Hdr
                | Self::Astc12x12Hdr
        )
    }

    pub fn is_hdr(&self) -> bool {
        matches!(
            self,
            Self::Astc4x4Hdr
                | Self::Astc5x4Hdr
                | Self::Astc5x5Hdr
                | Self::Astc6x5Hdr
                | Self::Astc6x6Hdr
                | Self::Astc8x5Hdr
                | Self::Astc8x6Hdr
                | Self::Astc8x8Hdr
                | Self::Astc10x5Hdr
                | Self::Astc10x6Hdr
                | Self::Astc10x8Hdr
                | Self::Astc10x10Hdr
                | Self::Astc12x10Hdr
                | Self::Astc12x12Hdr
        )
    }
}
