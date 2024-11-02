pub mod converter;
pub mod format;

use std::io::SeekFrom;

use binrw::binread;

#[binread]
// #[br(magic = b"\x02\x02\x02\x01")]
#[derive(Debug)]
#[repr(C)]
pub struct TextureHeader {
    pub magic: [u8; 4],
    pub unk0: u8,
    pub format: format::PixelFormat,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u32,
    pub width: u16,
    pub height: u16,
    pub unk4: [u8; 16],
    pub data_size: u32,
    pub unk5: u16,
    pub mip_levels: u16,

    #[br(count = mip_levels as usize)]
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
