use std::fs;
use std::process::Command;

use crate::config::MountifyConfig;
use crate::util;

pub fn run() -> anyhow::Result<()> {
    let config = MountifyConfig::load_or_default(util::PERSISTENT_DIR, util::MODDIR);

    // stop; start restart android
    if config.mountify_stop_start {
        let _ = Command::new("stop").status();
        let _ = Command::new("start").status();
    }

    // handle kernel umount
    let mount_list_path = format!("{}/mountify_mount_list", util::LOG_FOLDER);

    match config.mountify_custom_umount {
        1 => do_susfs_umount(&mount_list_path),
        2 => do_ksud_umount(&mount_list_path),
        _ => {}
    }

    // generate diff
    let before = format!("{}/before", util::LOG_FOLDER);
    let after = format!("{}/after", util::LOG_FOLDER);

    let diff_output = Command::new("busybox")
        .arg("diff")
        .arg(&before)
        .arg(&after)
        .output()
        .ok();

    if let Some(output) = diff_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let fs_alias = &config.fs_type_alias;

        let diff: Vec<String> = stdout
            .lines()
            .filter(|l| l.contains(fs_alias))
            .map(|l| l.to_string())
            .collect();

        if !diff.is_empty() {
            let diff_content = diff.join("\n");
            let _ = fs::write(format!("{}/mount_diff", util::MODDIR), &diff_content);
        }
    }

    Ok(())
}

fn do_susfs_umount(mount_list_path: &str) {
    if let Ok(content) = fs::read_to_string(mount_list_path) {
        for mount in content.lines() {
            let mount = mount.trim();
            if mount.is_empty() {
                continue;
            }
            // oplus workaround
            if mount.contains("/my_") {
                let _ = Command::new("/data/adb/ksu/bin/ksu_susfs")
                    .arg("add_try_umount")
                    .arg(format!("/mnt/vendor{}", mount))
                    .arg("1")
                    .status();
            }
            let _ = Command::new("/data/adb/ksu/bin/ksu_susfs")
                .arg("add_try_umount")
                .arg(mount)
                .arg("1")
                .status();
        }
    }
}

fn do_ksud_umount(mount_list_path: &str) {
    if let Ok(content) = fs::read_to_string(mount_list_path) {
        for mount in content.lines() {
            let mount = mount.trim();
            if mount.is_empty() {
                continue;
            }
            let _ = Command::new("/data/adb/ksud")
                .arg("kernel")
                .arg("umount")
                .arg("add")
                .arg(mount)
                .arg("--flags")
                .arg("2")
                .status();
        }
    }

    let _ = Command::new("/data/adb/ksud")
        .arg("kernel")
        .arg("notify-module-mounted")
        .status();
}
