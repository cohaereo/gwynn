use std::{collections::BTreeMap, path::Path};

use hashbrown::HashMap;

use crate::filetype::FileType;

#[derive(Debug)]
pub struct Directory {
    pub name: String,
    pub files: BTreeMap<String, FileEntry>,
    pub subdirectories: BTreeMap<String, Directory>,
}

impl Directory {
    pub fn new_root() -> Self {
        Self {
            name: String::new(),
            files: Default::default(),
            subdirectories: Default::default(),
        }
    }

    pub fn add_file(&mut self, file: FileEntry, path: &Path) {
        if path.components().count() == 1 {
            self.files.insert(file.name.clone(), file);
        } else {
            // We know there's at least one component because of the check above.
            let next_dir = path
                .components()
                .next()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .to_string();
            // Remove the first component from the path.
            let next_path = path.strip_prefix(&next_dir).unwrap();

            self.subdirectories
                .entry(next_dir.clone())
                .or_insert_with(|| Directory {
                    name: next_dir,
                    files: Default::default(),
                    subdirectories: Default::default(),
                })
                .add_file(file, next_path);
        }
    }
}

#[derive(Debug)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub data_file: String,
    pub ftype: FileType,
    pub info: gwynn_mpk::EntryHeader,
}
