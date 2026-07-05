use std::fs;
use std::path::Path;

pub const PERSISTENT_DIR: &str = "/data/adb/mountify";
pub const MODDIR: &str = "/data/adb/modules/mountify-rs";
pub const LOG_FOLDER: &str = "/dev/mountify_logs";
pub const MOUNTIFY_LOCK: &str = "/dev/mountify_single_instance";

pub const TARGETS: &[&str] = &[
    "odm", "product", "system_ext", "vendor", "apex", "mi_ext",
    "my_bigball", "my_carrier", "my_company", "my_engineering",
    "my_heytap", "my_manifest", "my_preload", "my_product",
    "my_region", "my_reserve", "my_stock", "oem", "optics", "prism",
];

pub const DECOY_FOLDER_CANDIDATES: &[&str] = &[
    "/oem", "/second_stage_resources", "/patch_hw", "/postinstall",
    "/system_dlkm", "/oem_dlkm", "/acct",
];

pub fn ensure_dir(path: &str) -> anyhow::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn read_file(path: &str) -> anyhow::Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn write_file(path: &str, content: &str) -> anyhow::Result<()> {
    fs::write(path, content)?;
    Ok(())
}

pub fn dmesg(msg: &str) {
    if let Ok(f) = fs::OpenOptions::new().write(true).open("/dev/kmsg") {
        use std::io::Write;
        let _ = writeln!(&f, "mountify: {}", msg);
    }
}

pub fn is_writable(path: &str) -> bool {
    let cpath = std::ffi::CString::new(path).ok();
    match cpath {
        Some(p) => unsafe { libc::access(p.as_ptr(), libc::W_OK) == 0 },
        None => false,
    }
}

pub fn find_mnt_folder() -> Option<String> {
    let mut mnt = None;
    if is_writable("/mnt") { mnt = Some("/mnt".into()) }
    if is_writable("/mnt/vendor") && !mount_has("/mnt/vendor") { mnt = Some("/mnt/vendor".into()) }
    mnt
}

pub fn mount_has(path: &str) -> bool {
    let content = fs::read_to_string("/proc/mounts").unwrap_or_default();
    content.lines().any(|l| l.contains(&format!(" {} ", path)))
}

pub fn fs_type_available(fstype: &str) -> bool {
    let content = fs::read_to_string("/proc/filesystems").unwrap_or_default();
    content.lines().any(|l| l.contains(fstype))
}

pub fn is_overlay_available() -> bool {
    fs_type_available("overlay")
}
