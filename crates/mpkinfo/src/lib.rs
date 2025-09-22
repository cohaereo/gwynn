use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
pub struct ResourcesHeader {
    #[br(assert(version == 2, "Only version 2 is supported"))]
    pub version: u32,
    #[br(temp)]
    pub record_num: u32,
    #[br(count = record_num)]
    pub records: Vec<ResourceEntry>,
}

#[binread]
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    pub asset_size: u32,
    pub flags: u32,
    pub unk: u8,
    #[br(map = |s: [u8; 3]| String::from_utf8_lossy(&s).to_string())]
    pub extension: String,
    pub hash: u32,
    pub offset: u32,
}

impl ResourceEntry {
    pub fn is_directory(&self) -> bool {
        self.flags & 1 != 0
    }

    pub fn file_number(&self) -> usize {
        (self.flags >> 1) as usize
    }
}
