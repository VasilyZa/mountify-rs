use std::fs;
use std::path::Path;

use crate::install as check;
use crate::util;

pub fn run(modpath: Option<String>) -> anyhow::Result<()> {
    let modpath = modpath.unwrap_or_else(|| "/data/adb/modules/mountify-rs".to_string());

    // test overlayfs
    check::check_overlayfs()?;
    println!("[+] CONFIG_OVERLAY_FS");
    println!("[+] overlay found in /proc/filesystems");

    // find MNT_FOLDER
    let mnt_folder = util::find_mnt_folder()
        .ok_or_else(|| anyhow::anyhow!("no writable mount folder found"))?;

    // test tmpfs xattr
    let has_tmpfs_xattr = check::test_tmpfs_xattr(&mnt_folder)?;

    if has_tmpfs_xattr {
        println!("[+] CONFIG_TMPFS_XATTR");
        println!("[+] tmpfs extended attribute test passed");
    } else {
        println!("[!] CONFIG_TMPFS_XATTR fail!");
        println!("[+] testing for ext4 sparse image fallback mode");

        if check::check_tools() {
            check::test_ext4_sparse(&mnt_folder, check::is_ksu())?;
            println!("[+] ext4 sparse fallback mode enabled");
        } else {
            return Err(anyhow::anyhow!("tools not found, bail out."));
        }
    }

    // copy config files
    let persistent_dir = util::PERSISTENT_DIR;
    util::ensure_dir(persistent_dir)?;

    let configs = ["modules.txt", "whiteouts.txt", "config.sh"];
    for file in &configs {
        let persistent_file = format!("{}/{}", persistent_dir, file);
        if !Path::new(&persistent_file).exists() {
            let mod_file = format!("{}/{}", modpath, file);
            if Path::new(&mod_file).exists() {
                let content = fs::read_to_string(&mod_file)?;
                fs::write(&persistent_file, &content)?;
                println!("[+] moving {}", file);
            }
        }
    }

    Ok(())
}
