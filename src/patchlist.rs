use std::collections::{HashMap, HashSet};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct PatchList {
    // TODO(cohae): a filehash is a 2-element array containing a string+integer for the md5 hash and size respectively (because fuck using structures i guess???)
    // pub android64_common: HashMap<String, FileHash>,
    // pub android_emulator: HashMap<String, FileHash>,
    // pub common: HashMap<String, FileHash>,
    pub file_sections: Vec<String>,
    pub huge_split_map: HashMap<String, usize>,
    pub mapping_ignorelist: HashSet<String>,

    /// A date/time string, formatted as "YYYYMMDDhhmm" (eg. "202505252344")
    pub mapping_version: String,

    pub mpk_exclude: HashSet<String>,

    pub package_tags_list: HashMap<String, Vec<String>>,
    pub package_tags_sections: Vec<String>,
    pub package_to_subtags: HashMap<String, Vec<String>>,
}
