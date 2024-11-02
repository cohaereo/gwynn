pub mod compression;

use binrw::binread;

#[binread]
#[derive(Debug)]
pub struct EntryHeader {
    _path_len: u32,
    #[br(count = _path_len)]
    _path_bytes: Vec<u8>,
    #[br(calc(String::from_utf8_lossy(&_path_bytes).to_string()))]
    pub path: String,
    pub asset_id: u64,
    pub length: u64,
    pub index: u16,
    _hash_bytes: [u8; 32],
    #[br(calc(String::from_utf8_lossy(&_hash_bytes).to_string()))]
    pub hash: String,
    pub flags: u16,
    pub offset: u64,
}

impl EntryHeader {
    pub fn is_directory(&self) -> bool {
        self.flags & 1 != 0
    }
}
