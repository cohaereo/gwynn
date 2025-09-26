use std::{collections::HashMap, io::Read};

use anyhow::Context;
use binrw::BinReaderExt;
use ext4::Ext4Reader;
use unix_path::{Path as UnixPath, PathBuf as UnixPathBuf};

use crate::{filetype::FileType, sources::mumuplayer::MumuPlayer};

pub mod apk;
pub mod filetype;
pub mod sources;

pub struct Filesystem {
    ext4: Ext4Reader<vdi::slice::OwnedSlice>,

    patch_basepath: UnixPathBuf,
    patch_paths: Vec<UnixPathBuf>,

    paths: HashMap<UnixPathBuf, FilePointer>,
    paths_by_filetype: HashMap<FileType, Vec<UnixPathBuf>>,
}

impl Filesystem {
    pub fn ext4(&self) -> &Ext4Reader<vdi::slice::OwnedSlice> {
        &self.ext4
    }

    pub fn read_path<P: AsRef<UnixPath>>(&self, path: P) -> anyhow::Result<Vec<u8>> {
        todo!()
    }

    // pub fn read_uuid(&self, uuid: Uuid) -> anyhow::Result<Vec<u8>> {
    //     todo!()
    // }

    pub fn iter_by_type(&self, filetype: FileType) -> impl Iterator<Item = &UnixPathBuf> {
        self.paths_by_filetype
            .get(&filetype)
            .map(|v| v.iter())
            .into_iter()
            .flatten()
    }

    pub fn iter_types(&self) -> impl Iterator<Item = (FileType, &Vec<UnixPathBuf>)> {
        self.paths_by_filetype.iter().map(|(k, v)| (*k, v))
    }

    pub fn iter_paths(&self) -> impl Iterator<Item = &UnixPathBuf> {
        self.paths.keys()
    }

    pub fn iter_patch_paths(&self) -> impl Iterator<Item = &UnixPathBuf> {
        self.patch_paths.iter()
    }
}

impl Filesystem {
    pub fn open() -> anyhow::Result<Self> {
        let Some(vm) =
            MumuPlayer::open_biggest_vm_ext4().context("Failed to open mumuplayer VM")?
        else {
            anyhow::bail!("No MuMuPlayer VMs found");
        };

        Self::open_from_ext4(vm)
    }

    pub fn open_from_ext4(ext4: Ext4Reader<vdi::slice::OwnedSlice>) -> anyhow::Result<Self> {
        let patch_basepath =
            UnixPathBuf::from("/media/0/Android/data/com.netease.g108na/files/LocalData/Patch/");

        let mut patch_paths = vec![];
        let mut paths = HashMap::new();
        let mut paths_by_filetype: HashMap<FileType, Vec<UnixPathBuf>> = HashMap::new();
        for i in 0.. {
            let filename = match i {
                0 => "Patch.mpkinfo",
                _ => &format!("Patch{i}.mpkinfo"),
            };
            let patch_path = patch_basepath.join(filename);
            if !ext4.exists(&patch_path) {
                break;
            }

            patch_paths.push(patch_path.clone());

            let mut buf = vec![];
            ext4.open(&patch_path)?.read_to_end(&mut buf)?;
            let mut cursor = std::io::Cursor::new(&buf);
            loop {
                let entry = match cursor.read_le::<gwynn_mpk::EntryHeader>() {
                    Ok(o) => o,
                    Err(e) => {
                        if e.is_eof() {
                            break;
                        }

                        return Err(e.into());
                    }
                };

                if entry.flags & 1 != 0 {
                    // This is a directory entry, skip it.
                    continue;
                }

                let path = UnixPathBuf::from(&entry.path);
                paths.insert(
                    path.clone(),
                    FilePointer::Patch {
                        index: i,
                        offset: entry.offset,
                        size: entry.length as usize,
                    },
                );

                let filetype = FileType::guess_from_path(&entry.path).unwrap_or(FileType::Unknown);
                paths_by_filetype.entry(filetype).or_default().push(path);
            }
        }

        Ok(Self {
            ext4,
            patch_basepath,
            patch_paths,

            paths,
            paths_by_filetype,
        })
    }
}

enum FilePointer {
    Resource {
        index: usize,
        offset: u64,
        size: usize,
    },
    Patch {
        index: usize,
        offset: u64,
        size: usize,
    },
}
