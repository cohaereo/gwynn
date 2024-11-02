pub mod compression;

use binrw::binread;

#[binread]
#[derive(Debug)]
pub struct EntryHeader {
    #[br(temp)]
    path_len: u32,
    #[br(map = |v: Vec<u8>| String::from_utf8_lossy(&v).to_string(), count = path_len as usize)]
    pub path: String,
    pub asset_id: u64,
    pub length: u64,
    pub index: u16,
    #[br(map = |v: [u8; 32]| String::from_utf8_lossy(&v).to_string())]
    pub hash: String,
    pub flags: u16,
    pub offset: u64,
}

impl EntryHeader {
    pub fn is_directory(&self) -> bool {
        self.flags & 1 != 0
    }
}
