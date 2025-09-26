use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let fs = gwynn_fs::Filesystem::open().context("failed to open filesystem")?;
    let apps = gwynn_fs::apk::scan_for_apps(fs.ext4()).context("Failed to scan for apps")?;
    let app = apps
        .iter()
        .find(|app| app.package_name == "com.netease.g108na")
        .context("Destiny: Rising not installed")?;

    println!(
        "Found Destiny: Rising installation at {}",
        app.path.display()
    );
    println!("Split packs:");
    for config in &app.split_packs {
        let meta = fs.ext4().metadata(app.split_pack_path(config)).unwrap();
        println!(
            "  {} ({}MB)",
            app.split_pack_path(config).display(),
            meta.size / 1_000_000
        );
    }

    println!();
    println!("Patches:");
    for path in fs.iter_patch_paths() {
        let meta = fs.ext4().metadata(path.with_extension("mpk")).unwrap();
        println!("  {} ({}MB)", path.display(), meta.size / 1_000_000);
    }

    println!();
    println!("File types:");
    for (filetype, paths) in fs.iter_types() {
        println!("  {:?} - {} files", filetype, paths.len());
    }

    Ok(())
}
