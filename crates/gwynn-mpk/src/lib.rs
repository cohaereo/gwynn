pub mod compression;
pub mod indexer;

use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
pub struct EntryHeader {
    #[br(temp)]
    path_len: u32,
    #[br(map = |v: Vec<u8>| decode_path(&v), count = path_len as usize, dbg)]
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

fn decode_path(bytes: &[u8]) -> String {
    if bytes.len() <= 2
        || ((bytes[0] as char).is_alphanumeric()
            && (bytes[1] as char).is_alphanumeric()
            && bytes[2] == b'/')
    {
        // If the first three bytes are alphanumeric followed by a '/', it's a nameless path and we dont need to decrypt it
        String::from_utf8_lossy(bytes).to_string()
    } else {
        let part_size = bytes.len() % 7;
        let mut decoded = String::new();
        for b in 0..part_size {
            let byte = bytes[b];
            let decoded_byte = (byte) ^ 0x2B;
            decoded.push(decoded_byte as char);
        }

        for b in part_size..bytes.len() {
            let byte = bytes[b];
            let decoded_byte = (byte) ^ 0x35;
            decoded.push(decoded_byte as char);
        }

        decoded
    }
}
