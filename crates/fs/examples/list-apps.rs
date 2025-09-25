// Iterates through all supported emulator filesystems and lists installed APKs

use anyhow::Context;
use gwynn_fs::sources::mumuplayer::MumuPlayer;

fn main() -> anyhow::Result<()> {
    match MumuPlayer::iter_vms() {
        Ok(vms) => {
            for vm in vms {
                println!("Found MuMuPlayer VM: {}", vm.display());
                let fs = MumuPlayer::open_vm_ext4(&vm)
                    .context("failed to open ext4 filesystem")?
                    .expect("image does not contain an ext4 filesystem");
                let apps = gwynn_fs::apk::scan_for_apps(&fs).context("Failed to scan for apps")?;
                for app in apps {
                    println!("  Found installed app: {}", app.package_name);
                    println!("    Split configs: {:?}", app.split_configs);
                    println!("    Split packs: {:?}", app.split_packs);
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing MuMuPlayer VMs: {e}");
        }
    }

    Ok(())
}
