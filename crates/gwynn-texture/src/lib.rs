pub mod converter;
pub mod format;

use std::io::SeekFrom;

use binrw::binread;
use format::PixelFormat;

#[binread]
#[derive(Debug)]
#[repr(C)]
pub struct TextureHeader {
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
    pub mip_filter: SamplerFilter,
    pub address_u: SampleAddress,
    pub address_v: SampleAddress,
    pub format: PixelFormat,
    pub mip_level: u8,
    pub flags: u8,
    pub compression_preset: TextureCompression,
    pub lod_group: TextureLodGroup,
    pub mip_gen_preset: TextureMipGen,
    pub texture_type: TextureType,
    pub width: u16,
    pub height: u16,
    pub default_color: [f32; 4],
    pub size: u32,
    pub unk: u16,
    pub mip_count: u16,

    #[br(count = mip_count as usize)]
    pub mips: Vec<MipHeader>,
}

#[binread]
#[derive(Debug)]
pub struct MipHeader {
    #[br(temp)]
    data_start: binrw::PosValue<()>,

    /// Total mip data size, header + (compressed) data
    pub total_size: u32,

    pub width: u16,
    pub height: u16,
    pub unk0: u16,
    pub unk1: u16,
    /// ⚠ This is the length of the uncompressed data. The compression magic/header are not included in this number
    pub data_size: u32,
    pub data_offset: binrw::PosValue<()>,

    #[br(seek_before(SeekFrom::Start(data_start.pos + total_size as u64)))]
    pub data_end: binrw::PosValue<()>,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum SamplerFilter {
    None = 0,
    Point = 1,
    Linear = 2,
    Anisotropic = 3,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum SampleAddress {
    None = 0,
    Wrap = 1,
    Mirror = 2,
    Clamp = 3,
    FromTexture = 4,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum SampleQuality {
    None = 0,
    Sample2x = 1,
    Sample4x = 2,
    Sample8x = 3,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum TextureType {
    Texture1D = 0,
    Texture2D = 1,
    Texture3D = 2,
    Cube = 3,
    Texture2DArray = 4,
    CubeArray = 5,
    Array = 6,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum TextureCompression {
    Default = 0,
    NormalMap = 1,
    DisplacementMap = 2,
    Grayscale = 3,
    HDR = 4,
    NormalMapUncompress = 5,
    NormalMapBC5 = 6,
    VectorMap = 7,
    Uncompressed = 8,
    LightMap = 9,
    EnvMap = 10,
    MixMap = 11,
    UI = 12,
    TerrainBlock = 13,
    TerrainIndex = 14,
    NormalMapCompact = 15,
    BC6H = 16,
    BC7 = 17,
    LightProfile = 18,
    LUTHDR = 19,
    LUTLOG = 20,
    TerrainNormalMap = 21,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum TextureLodGroup {
    World = 0,
    WorldNormalMap = 1,
    WorldSpecular = 2,
    Character = 3,
    CharacterNormalMap = 4,
    CharacterSpecular = 5,
    Weapon = 6,
    WeaponNormalMap = 7,
    WeaponSpecular = 8,
    Cinematic = 9,
    Effect = 10,
    EffectUnfiltered = 11,
    Sky = 12,
    Gui = 13,
    RenderTarget = 14,
    ShadowMap = 15,
    LUT = 16,
    TerrainBlockMap = 17,
    TerrainIndexMap = 18,
    TerrainLightMap = 19,
    ImageBaseReflection = 20,
}

#[binread]
#[br(repr(u8))]
#[derive(Debug)]
pub enum TextureMipGen {
    FromTextureGroup = 0,
    Simple = 1,
    Sharpen = 2,
    NoMip = 3,
    Blur = 4,
    AlphaDistribution = 5,
}
