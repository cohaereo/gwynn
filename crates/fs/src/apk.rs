use std::collections::HashSet;

use base64::Engine;
use bootsector::pio::ReadAt;
use ext4::Ext4Reader;
use log::error;

pub struct InstalledApp {
    pub package_name: String,
    pub path: unix_path::PathBuf,
    pub signature: [u8; 16],

    pub split_configs: HashSet<String>,
    pub split_packs: HashSet<String>,
}

impl InstalledApp {
    pub fn base_apk_path(&self) -> unix_path::PathBuf {
        self.path.join("base.apk")
    }

    pub fn split_config_path(&self, config: &str) -> unix_path::PathBuf {
        self.path.join(format!("split_config.{}.apk", config))
    }

    pub fn split_pack_path(&self, pack: &str) -> unix_path::PathBuf {
        self.path.join(format!("{}.apk", pack))
    }
}

pub fn scan_for_apps<R: ReadAt>(fs: &Ext4Reader<R>) -> anyhow::Result<Vec<InstalledApp>> {
    let mut app_dirs: Vec<ext4::DirectoryEntry> = vec![];
    let mut add_dir = |path: &str| {
        if let Ok(entries) = fs.read_dir(path) {
            app_dirs.extend(entries);
        }
    };
    add_dir("/data/app");
    add_dir("/app");

    let mut apps = Vec::new();
    for session_dir in app_dirs {
        if !session_dir.is_dir || !session_dir.name.starts_with("~~") {
            continue;
        }

        for package_dir in fs.read_dir(&session_dir.path)? {
            if !package_dir.is_dir {
                continue;
            }

            let parts = package_dir.name.split("-").collect::<Vec<_>>();
            if parts.len() != 2 {
                error!("Invalid app dir name: {}", package_dir.name);
                continue;
            }

            let package_name = parts[0].to_string();
            let signature = base64::prelude::BASE64_URL_SAFE.decode(parts[1])?;
            if signature.len() != 16 {
                error!(
                    "Invalid signature length for app {}: {}",
                    package_name,
                    signature.len()
                );
                continue;
            }

            let signature: [u8; 16] = signature
                .try_into()
                .expect("unreachable: length is checked");

            let mut split_configs = HashSet::new();
            let mut split_packs = HashSet::new();
            for entry in fs.read_dir(&package_dir.path)? {
                if entry.is_file
                    && entry.name.starts_with("split_config.")
                    && entry.name.ends_with(".apk")
                {
                    let config = entry.name["split_config.".len()..entry.name.len() - ".apk".len()]
                        .to_string();
                    split_configs.insert(config);
                }

                if entry.is_file
                    && entry.name.starts_with("split_pack")
                    && entry.name.ends_with(".apk")
                {
                    split_packs.insert(entry.name.trim_end_matches(".apk").to_string());
                }
            }

            apps.push(InstalledApp {
                package_name,
                path: package_dir.path,
                signature,
                split_configs,
                split_packs,
            });
        }
    }

    Ok(apps)
}
