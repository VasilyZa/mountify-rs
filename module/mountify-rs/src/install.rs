use std::fs;
use std::path::Path;

use crate::util;

pub fn check_overlayfs() -> anyhow::Result<()> {
    if util::is_overlay_available() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("CONFIG_OVERLAY_FS is required!"))
    }
}

pub fn test_tmpfs_xattr(mnt_folder: &str) -> anyhow::Result<bool> {
    use std::ffi::CString;

    let testfile = format!("{}/tmpfs_xattr_testfile", mnt_folder);
    let _ = fs::remove_file(&testfile);

    let cpath = CString::new(testfile.as_str())
        .map_err(|_| anyhow::anyhow!("invalid path"))?;

    // mknod(path, S_IFCHR | 0644, makedev(0, 0))
    let ret = unsafe {
        libc::mknod(
            cpath.as_ptr(),
            libc::S_IFCHR | 0o644,
            libc::makedev(0, 0),
        )
    };

    if ret != 0 {
        let _ = fs::remove_file(&testfile);
        return Ok(false);
    }

    // test trusted.overlay.whiteout xattr
    let cname = CString::new("trusted.overlay.whiteout").unwrap();
    let val = b"y";
    let xret = unsafe {
        libc::setxattr(
            cpath.as_ptr(),
            cname.as_ptr(),
            val.as_ptr() as *const libc::c_void,
            val.len(),
            0,
        )
    };

    let _ = fs::remove_file(&testfile);

    Ok(xret == 0)
}

pub fn test_ext4_sparse(mnt_folder: &str, is_ksu: bool) -> anyhow::Result<()> {
    // on 4.x+ kernels with ext4 support we don't need to test
    let ver = fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    if let Some(major_str) = ver.split('.').next() {
        if let Ok(major) = major_str.parse::<u32>() {
            if major >= 4 && util::fs_type_available("ext4") {
                return Ok(());
            }
        }
    }

    // fallback test: create a sparse image, format, mount, umount
    let test_image = format!("{}/mountify-ext4-test", mnt_folder);
    let test_mount = format!("{}/mountify-mount-test", mnt_folder);

    let _ = fs::remove_file(&test_image);
    let _ = fs::remove_dir_all(&test_mount);
    fs::create_dir_all(&test_mount)?;

    // create sparse with dd-style truncation
    let f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&test_image)?;
    f.set_len(8 * 1024 * 1024)?;
    drop(f);

    let status = std::process::Command::new("/system/bin/mkfs.ext4")
        .arg("-O")
        .arg("^has_journal")
        .arg(&test_image)
        .status()
        .map_err(|e| anyhow::anyhow!("mkfs.ext4: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&test_image);
        let _ = fs::remove_dir_all(&test_mount);
        return Err(anyhow::anyhow!("mkfs.ext4 failed"));
    }

    if is_ksu {
        let _ = std::process::Command::new("busybox")
            .arg("chcon")
            .arg("u:object_r:ksu_file:s0")
            .arg(&test_image)
            .status();
    }

    // mount the image
    let mount_status = std::process::Command::new("busybox")
        .arg("mount")
        .arg("-o")
        .arg("loop,rw")
        .arg(&test_image)
        .arg(&test_mount)
        .status()
        .map_err(|e| anyhow::anyhow!("mount: {}", e))?;

    let _ = std::process::Command::new("busybox")
        .arg("umount")
        .arg("-l")
        .arg(&test_mount)
        .status();

    let _ = fs::remove_file(&test_image);
    let _ = fs::remove_dir_all(&test_mount);

    if !mount_status.success() {
        return Err(anyhow::anyhow!("ext4 fallback mode test fail"));
    }

    Ok(())
}

pub fn check_tools() -> bool {
    Path::new("/system/bin/mkfs.ext4").exists()
        && Path::new("/system/bin/resize2fs").exists()
}

pub fn is_ksu() -> bool {
    util::file_exists("/data/adb/ksu/bin")
}

pub fn is_apatch() -> bool {
    util::file_exists("/data/adb/ap/bin")
}

pub fn is_magisk() -> bool {
    util::file_exists("/data/adb/magisk")
}

pub fn get_ksu_version() -> u32 {
    fs::read_to_string("/data/adb/ksu/version_code")
        .ok()
        .and_then(|c| c.trim().parse().ok())
        .unwrap_or(0)
}

pub fn has_ksud_nuke_ext4() -> bool {
    if !util::file_exists("/data/adb/ksud") {
        return false;
    }
    let out = std::process::Command::new("/data/adb/ksud")
        .arg("kernel")
        .output()
        .ok();
    out.and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("nuke-ext4-sysfs"))
        .unwrap_or(false)
}

pub fn check_susfs_conflict() -> bool {
    let susfs_bin = "/data/adb/ksu/bin/ksu_susfs";
    if !util::file_exists(susfs_bin) {
        return false;
    }

    let out = std::process::Command::new(susfs_bin)
        .arg("show")
        .arg("version")
        .output()
        .ok();

    if let Some(output) = out {
        let ver_str = String::from_utf8_lossy(&output.stdout);
        let ver = ver_str
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches('v')
            .replace('.', "");
        let ver_num: u32 = ver.parse().unwrap_or(0);
        return ver_num == 1510 || ver_num == 1511;
    }
    false
}
