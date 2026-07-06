use std::fs;
use std::os::android::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::selinux;
use crate::util;

pub const BLACKLISTED_MODULES: &[&str] = &["De-bloater"];

pub fn should_mount_module(module_id: &str, _moddir: &str, is_metamodule: bool) -> bool {
    let target_dir = format!("/data/adb/modules/{}", module_id);
    let system_dir = format!("{}/system", target_dir);

    if !Path::new(&system_dir).exists() {
        return false;
    }
    if util::file_exists(&format!("{}/disable", target_dir)) {
        return false;
    }
    if util::file_exists(&format!("{}/remove", target_dir)) {
        return false;
    }
    if util::file_exists(&format!("{}/skip_mountify", target_dir)) {
        return false;
    }
    if util::file_exists(&format!("{}/system/etc/hosts", target_dir)) {
        return false;
    }
    if BLACKLISTED_MODULES.contains(&module_id) {
        util::dmesg(&format!("module {} is blacklisted", module_id));
        return false;
    }

    if is_metamodule && util::file_exists(&format!("{}/skip_mount", target_dir)) {
        util::dmesg(&format!("module {} has skip_mount", module_id));
        return false;
    }

    true
}

pub fn handle_skip_mount(module_id: &str, _moddir: &str, is_metamodule: bool) -> anyhow::Result<()> {
    let target_dir = format!("/data/adb/modules/{}", module_id);
    let skip_path = format!("{}/skip_mount", target_dir);
    let persistent_skip = format!("{}/skipped_modules", util::PERSISTENT_DIR);

    let is_litemode = util::file_exists("/data/adb/.litemode_enable")
        && std::env::var("APATCH_BIND_MOUNT").unwrap_or_default() == "true";

    if is_litemode || is_metamodule {
        if util::file_exists(&skip_path) {
            let _ = fs::remove_file(&skip_path);
        }
        if util::file_exists(&persistent_skip) {
            let _ = fs::remove_file(&persistent_skip);
        }
    } else if !util::file_exists(&skip_path) {
        fs::write(&skip_path, "")?;
        fs::write(&persistent_skip, format!("{}\n", module_id))?;
    }
    Ok(())
}

pub fn copy_module_to_fake_mount(
    module_id: &str,
    mnt_folder: &str,
    fake_name: &str,
) -> anyhow::Result<()> {
    let base_dir = format!("/data/adb/modules/{}/system", module_id);
    let dest_dir = format!("{}/{}", mnt_folder, fake_name);

    let base = Path::new(&base_dir);
    let dest = Path::new(&dest_dir);

    cp_r(base, dest)?;
    selinux::mirror_selinux_contexts(&base_dir, &dest_dir)?;
    selinux::mirror_opaque_dirs(&base_dir, &dest_dir)?;

    util::dmesg(&format!("module {} copied to {}", module_id, dest_dir));
    Ok(())
}

fn cp_r(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(src).follow_links(true) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src)
            .map_err(|_| anyhow::anyhow!("strip_prefix failed"))?;
        let dest_path = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
            let mode = entry.metadata()?.st_mode();
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(mode));
        } else if entry.file_type().is_symlink() {
            let target = fs::read_link(entry.path())?;
            let _ = fs::remove_file(&dest_path);
            std::os::unix::fs::symlink(&target, &dest_path)?;
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), &dest_path)?;
            let mode = entry.metadata()?.st_mode();
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(mode));
        }
    }
    Ok(())
}
