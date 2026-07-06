use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MountifyConfig {
    pub mountify_mounts: u8,
    pub fake_mount_name: String,
    pub test_decoy_mount: bool,
    pub mountify_stop_start: bool,
    pub fs_type_alias: String,
    pub mount_device_name: String,
    pub mountify_custom_umount: u8,
    pub mountify_expert_mode: bool,
    pub use_ext4_sparse: bool,
    pub spoof_sparse: bool,
    pub fake_apex_name: String,
    pub sparse_size: u64,
    pub enable_lkm_nuke: bool,
    pub lkm_filename: String,
}

impl Default for MountifyConfig {
    fn default() -> Self {
        Self {
            mountify_mounts: 2,
            fake_mount_name: String::from("mountify"),
            test_decoy_mount: false,
            mountify_stop_start: false,
            fs_type_alias: String::from("overlay"),
            mount_device_name: String::from("overlay"),
            mountify_custom_umount: 0,
            mountify_expert_mode: false,
            use_ext4_sparse: false,
            spoof_sparse: false,
            fake_apex_name: String::from("com.android.mntservice"),
            sparse_size: 2048,
            enable_lkm_nuke: false,
            lkm_filename: String::from("nuke.ko"),
        }
    }
}

impl MountifyConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to read config {}: {}", path.as_ref().display(), e))?;
        Self::from_str(&content)
    }

    pub fn load_or_default(persistent_dir: &str, moddir: &str) -> Self {
        let config_path = format!("{}/config.sh", persistent_dir);
        if Path::new(&config_path).exists() {
            Self::from_file(&config_path).unwrap_or_default()
        } else {
            let fallback = format!("{}/config.sh", moddir);
            Self::from_file(&fallback).unwrap_or_default()
        }
    }
}

impl FromStr for MountifyConfig {
    type Err = anyhow::Error;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let mut kv = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let mut value = line[eq_pos + 1..].trim().to_string();
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    value = value[1..value.len() - 1].to_string();
                }
                kv.insert(key, value);
            }
        }

        Ok(Self {
            mountify_mounts: kv.get("mountify_mounts").and_then(|v| v.parse().ok()).unwrap_or(2),
            fake_mount_name: kv.get("FAKE_MOUNT_NAME").cloned().unwrap_or_else(|| String::from("mountify")),
            test_decoy_mount: kv.get("test_decoy_mount").map(|v| v == "1").unwrap_or(false),
            mountify_stop_start: kv.get("mountify_stop_start").map(|v| v == "1").unwrap_or(false),
            fs_type_alias: kv.get("FS_TYPE_ALIAS").cloned().unwrap_or_else(|| String::from("overlay")),
            mount_device_name: kv.get("MOUNT_DEVICE_NAME").cloned().unwrap_or_else(|| String::from("overlay")),
            mountify_custom_umount: kv.get("mountify_custom_umount").and_then(|v| v.parse().ok()).unwrap_or(0),
            mountify_expert_mode: kv.get("mountify_expert_mode").map(|v| v == "1").unwrap_or(false),
            use_ext4_sparse: kv.get("use_ext4_sparse").map(|v| v == "1").unwrap_or(false),
            spoof_sparse: kv.get("spoof_sparse").map(|v| v == "1").unwrap_or(false),
            fake_apex_name: kv.get("FAKE_APEX_NAME").cloned().unwrap_or_else(|| String::from("com.android.mntservice")),
            sparse_size: kv.get("sparse_size").and_then(|v| v.parse().ok()).unwrap_or(2048),
            enable_lkm_nuke: kv.get("enable_lkm_nuke").map(|v| v == "1").unwrap_or(false),
            lkm_filename: kv.get("lkm_filename").cloned().unwrap_or_else(|| String::from("nuke.ko")),
        })
    }
}
