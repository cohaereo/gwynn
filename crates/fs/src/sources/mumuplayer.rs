use std::path::{Path, PathBuf};

use anyhow::Context;
use ext4::Ext4Reader;

pub struct MumuPlayer;

impl MumuPlayer {
    fn base_path() -> anyhow::Result<PathBuf> {
        let appdata =
            std::env::var_os("APPDATA").context("APPDATA environment variable not set")?;
        let path = PathBuf::from(appdata)
            .join("Netease")
            .join("MuMuPlayerGlobal")
            .join("install_config.json");

        serde_json::from_reader::<std::fs::File, serde_json::Value>(std::fs::File::open(&path)?)
            .context("Failed to read MuMuPlayer install_config.json")?
            .get("product")
            .and_then(|v| v.get("install_dir").and_then(|v| v.as_str()))
            .map(PathBuf::from)
            .context("MuMuPlayer install_path not found or invalid")
    }

    fn vms_path() -> anyhow::Result<PathBuf> {
        Ok(Self::base_path()?.join("vms"))
    }

    pub fn iter_vms() -> anyhow::Result<impl Iterator<Item = PathBuf>> {
        let vms_path = Self::vms_path()?;
        if !vms_path.exists() {
            anyhow::bail!("MuMuPlayer vms path does not exist: {}", vms_path.display());
        }
        let entries = std::fs::read_dir(vms_path)?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path());
        Ok(entries)
    }

    pub fn get_biggest_vm() -> anyhow::Result<Option<PathBuf>> {
        let mut biggest: Option<(PathBuf, u64)> = None;
        for vm_path in Self::iter_vms()? {
            let mpk_path = vm_path.join("data.vdi");
            if mpk_path.exists() {
                let metadata = std::fs::metadata(&mpk_path)?;
                let size = metadata.len();
                if biggest.is_none() || size > biggest.as_ref().unwrap().1 {
                    biggest = Some((vm_path, size));
                }
            }
        }
        Ok(biggest.map(|(path, _)| path))
    }

    pub fn open_vm_ext4(path: &Path) -> anyhow::Result<Option<Ext4Reader<vdi::slice::OwnedSlice>>> {
        if !path.exists() {
            anyhow::bail!("MuMuPlayer VM path does not exist: {}", path.display());
        }
        let data_path = path.join("data.vdi");
        if !data_path.exists() {
            anyhow::bail!(
                "MuMuPlayer VM data.vdi does not exist: {}",
                data_path.display()
            );
        }

        let file = std::fs::File::open(data_path).context("Failed to open VDI file")?;
        let mut disk = vdi::VdiDisk::open(Box::new(file))?;
        let largest_partition =
            bootsector::list_partitions(&mut disk, &bootsector::Options::default())?
                .into_iter()
                .max_by_key(|p| p.len)
                .context("Disk has no partitions")?;

        let range =
            largest_partition.first_byte..(largest_partition.first_byte + largest_partition.len);
        let slice = disk.slice_owned(range)?;
        let ext4 = Ext4Reader::new(slice).context("Failed to open ext4 filesystem")?;
        Ok(Some(ext4))
    }

    pub fn open_biggest_vm_ext4() -> anyhow::Result<Option<Ext4Reader<vdi::slice::OwnedSlice>>> {
        let path = Self::get_biggest_vm()?;
        let Some(path) = path else {
            return Ok(None);
        };

        Self::open_vm_ext4(&path)
    }
}
