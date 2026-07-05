use std::fs;
use std::path::Path;

use crate::anti_bootloop;
use crate::config::MountifyConfig;
use crate::copy;
use crate::install;
use crate::lkm;
use crate::mount;
use crate::util;

pub fn run(moddir: &str, is_metamodule: bool) -> anyhow::Result<()> {
    // prefix detection
    let dmesg_prefix = if is_metamodule {
        "mountify/metamount"
    } else {
        "mountify/post-fs-data"
    };

    // single instance check
    if util::file_exists(util::MOUNTIFY_LOCK) {
        util::dmesg(&format!("{}: mountify already ran!", dmesg_prefix));
        return Ok(());
    }
    fs::write(util::MOUNTIFY_LOCK, "")?;

    // anti-bootloop
    let boot = anti_bootloop::BootCount::check();
    if boot.should_disable {
        util::dmesg("anti-bootloop triggered, exiting");
        return Err(anyhow::anyhow!("anti-bootloop triggered"));
    }

    util::dmesg(&format!("{}: start!", dmesg_prefix));

    // find MNT_FOLDER
    let mnt_folder = util::find_mnt_folder()
        .ok_or_else(|| anyhow::anyhow!("no writable mount folder found"))?;

    // create log folder
    util::ensure_dir(util::LOG_FOLDER)?;

    // log before state
    let before = fs::read_to_string("/proc/mounts").unwrap_or_default();
    fs::write(format!("{}/before", util::LOG_FOLDER), &before)?;

    // load config
    let config = MountifyConfig::load_or_default(util::PERSISTENT_DIR, moddir);

    // check if disabled
    if config.mountify_mounts == 0 {
        update_disabled_description(moddir);
        return Ok(());
    }

    // validate fake mount name
    if config.fake_mount_name == "persist" {
        util::dmesg("folder name 'persist' is not allowed!");
        return Err(anyhow::anyhow!("persist is not allowed as fake mount name"));
    }

    // check if fake folder already exists
    let fake_path = format!("{}/{}", mnt_folder, config.fake_mount_name);
    if !config.mountify_expert_mode && Path::new(&fake_path).exists() {
        util::dmesg(&format!("fake folder {} already exists!", &fake_path));
        return Err(anyhow::anyhow!("fake folder already exists"));
    }

    // determine fs type alias
    let fs_alias = if util::fs_type_available(&config.fs_type_alias) {
        &config.fs_type_alias
    } else {
        "overlay"
    };

    // decoy mount detection
    let mut decoy_folder = String::new();
    let mut decoy_enabled = false;

    if config.test_decoy_mount && !util::file_exists(&format!("{}/no_tmpfs_xattr", moddir)) {
        for candidate in util::DECOY_FOLDER_CANDIDATES {
            if let Ok(entries) = fs::read_dir(candidate) {
                if entries.count() == 0 {
                    decoy_folder = candidate.to_string();
                    decoy_enabled = true;
                    util::dmesg(&format!("decoy folder {}", decoy_folder));
                    break;
                }
            }
        }
    }

    // stage1: mount /mnt or /mnt/vendor as tmpfs
    mount::stage1_mount(&mnt_folder)?;

    // create fake mount directory
    util::ensure_dir(&fake_path)?;

    // stage2: mount the fake folder
    let no_tmpfs_xattr = util::file_exists(&format!("{}/no_tmpfs_xattr", moddir));
    let is_ext4 = no_tmpfs_xattr || config.use_ext4_sparse;

    if is_ext4 {
        mount::stage2_mount_ext4(&mnt_folder, &config.fake_mount_name, config.sparse_size, install::is_ksu())?;
    } else {
        mount::stage2_mount_tmpfs(&mnt_folder, &config.fake_mount_name)?;
    }

    // placeholder
    fs::write(format!("{}/placeholder", fake_path), "")?;

    // decoy mount if enabled
    if decoy_enabled && Path::new(&decoy_folder).exists() {
        if let Ok(entries) = fs::read_dir(&decoy_folder) {
            if entries.count() == 0 {
                util::dmesg(&format!("mounting {}", decoy_folder));
                let _ = mount::mount_tmpfs(&decoy_folder);
            }
        }
    }

    // determine mount mode and copy modules
    let modules: Vec<String> = if config.mountify_mounts == 1 {
        // manual mode
        let modules_txt_path = format!("{}/modules.txt", util::PERSISTENT_DIR);
        if Path::new(&modules_txt_path).exists() {
            let content = fs::read_to_string(&modules_txt_path).unwrap_or_default();
            content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
                .filter(|id| !id.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    } else {
        // auto mode: find all modules with system folder
        let modules_dir = "/data/adb/modules";
        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(modules_dir) {
            for entry in entries.flatten() {
                if entry.path().join("system").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        list.push(name.to_string());
                    }
                }
            }
        }
        list
    };

    // process each module
    let mut mounted_modules: Vec<String> = Vec::new();

    for module_id in &modules {
        if !copy::should_mount_module(module_id, moddir, is_metamodule) {
            continue;
        }

        util::dmesg(&format!("processing {}", module_id));

        // handle skip_mount
        let _ = copy::handle_skip_mount(module_id, moddir, is_metamodule);

        // copy module to fake mount
        if let Err(e) = copy::copy_module_to_fake_mount(module_id, &mnt_folder, &config.fake_mount_name) {
            util::dmesg(&format!("failed to copy module {}: {}", module_id, e));
            continue;
        }

        mounted_modules.push(module_id.clone());
        // log to tmpfs (for current session)
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/modules", util::LOG_FOLDER))
            .and_then(|f| {
                use std::io::Write;
                writeln!(&f, "{}", module_id)
            });
        // log to persistent storage (for WebUI)
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/mounted_modules", util::PERSISTENT_DIR))
            .and_then(|f| {
                use std::io::Write;
                writeln!(&f, "{}", module_id)
            });
    }

    // handle ext4 remount
    if is_ext4 {
        mount::stage2_remount_ext4(
            &mnt_folder,
            &config.fake_mount_name,
            config.spoof_sparse,
            &config.fake_apex_name,
        )?;
    }

    // perform overlay mounts
    perform_overlay_mounts(
        &mnt_folder,
        &config.fake_mount_name,
        fs_alias,
        &config.mount_device_name,
        decoy_enabled,
        &decoy_folder,
    )?;

    // unmount decoy
    if decoy_enabled && Path::new(&decoy_folder).exists() {
        util::dmesg(&format!("unmounting {}", decoy_folder));
        let _ = mount::umount_lazy(&decoy_folder);
    }

    // LKM nuke for ext4
    if is_ext4 && !config.spoof_sparse {
        let ksud_nuke = util::file_exists(&format!("{}/ksud_has_nuke_ext4", moddir));

        if ksud_nuke {
            let mnt = fs::canonicalize(&fake_path).ok();
            if let Some(m) = mnt {
                util::dmesg(&format!("stage2/ext4: ksud kernel nuke-ext4-sysfs {}", m.display()));
                let _ = lkm::ksud_nuke_ext4(m.to_str().unwrap_or(&fake_path));
            }
        } else if config.enable_lkm_nuke {
            let lkm_path = format!("{}/lkm/{}", moddir, config.lkm_filename);
            if Path::new(&lkm_path).exists() {
                let mnt = fs::canonicalize(&fake_path)
                    .unwrap_or_else(|_| Path::new(&fake_path).to_path_buf());
                let _ = lkm::load_nuke_lkm(&lkm_path, mnt.to_str().unwrap_or(&fake_path));
            }
        }
    }

    // unmount stage2 (unless spoof_sparse)
    if !config.spoof_sparse {
        mount::umount_fake_mount(&mnt_folder, &config.fake_mount_name)?;
    }

    // clean up sparse image
    if is_ext4 {
        let image_path = format!("{}/mountify-ext4", mnt_folder);
        let _ = fs::remove_file(&image_path);
    }

    // unmount stage1
    mount::stage1_umount(&mnt_folder)?;

    // generate description
    update_module_description(moddir, &config, &mounted_modules)?;

    // log after state
    let after = fs::read_to_string("/proc/mounts").unwrap_or_default();
    fs::write(format!("{}/after", util::LOG_FOLDER), &after)?;

    util::dmesg(&format!("{}: finished!", dmesg_prefix));
    Ok(())
}

fn update_disabled_description(moddir: &str) {
    let modprop = format!("{}/module.prop", moddir);
    if let Ok(content) = fs::read_to_string(&modprop) {
        let new_desc = "description=状态：已禁用";
        let new_content = content
            .lines()
            .map(|l| {
                if l.starts_with("description=") {
                    new_desc
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write(&modprop, &new_content);
    }
}

fn update_module_description(
    moddir: &str,
    config: &MountifyConfig,
    mounted_modules: &[String],
) -> anyhow::Result<()> {
    let mode_str = match config.mountify_mounts {
        1 => "手动",
        _ => "自动",
    };

    let fstype_str = if config.use_ext4_sparse {
        "ext4"
    } else {
        "tmpfs"
    };

    let count = mounted_modules.len();
    let description = if count > 0 {
        format!("模式：{} 文件系统：{} 已加载 {} 个模块", mode_str, fstype_str, count)
    } else {
        format!("模式：{} 文件系统：{} 未加载模块", mode_str, fstype_str)
    };

    let modprop_path = format!("{}/module.prop", moddir);

    if let Ok(content) = fs::read_to_string(&modprop_path) {
        let new_desc = format!("description={}", description);
        let new_content = content
            .lines()
            .map(|l| {
                if l.starts_with("description=") {
                    new_desc.as_str()
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&modprop_path, &new_content)?;
    }

    Ok(())
}

fn perform_overlay_mounts(
    mnt_folder: &str,
    fake_name: &str,
    fs_alias: &str,
    device_name: &str,
    decoy_enabled: bool,
    decoy_folder: &str,
) -> anyhow::Result<()> {
    let cwd = format!("{}/{}", mnt_folder, fake_name);

    // single_depth: mount subdirs of /system
    single_depth(&cwd, fs_alias, device_name, decoy_enabled, decoy_folder)?;

    // controlled_depth: mount target partition dirs
    for target in util::TARGETS {
        let target_path = format!("/{}", target);
        let system_target_path = format!("/system/{}", target);

        if Path::new(&target_path).is_symlink() && !Path::new(&system_target_path).is_symlink() {
            // legacy: mount at /system/
            controlled_depth(
                &cwd, target, "/system/",
                fs_alias, device_name, decoy_enabled, decoy_folder,
            )?;
        } else {
            // modern: mount at /
            controlled_depth(
                &cwd, target, "/",
                fs_alias, device_name, decoy_enabled, decoy_folder,
            )?;
        }
    }

    Ok(())
}

fn single_depth(
    cwd: &str,
    fs_alias: &str,
    device_name: &str,
    decoy_enabled: bool,
    decoy_folder: &str,
) -> anyhow::Result<()> {
    let skip_dirs = ["odm", "product", "system_ext", "vendor"];

    if let Ok(entries) = fs::read_dir(cwd) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }

            let name = entry.file_name();
            let name_str = name.to_str().unwrap_or("");

            // skip partition dirs handled in controlled_depth
            if skip_dirs.contains(&name_str) {
                continue;
            }

            let pwd = entry.path();
            let pwd_str = pwd.to_str().unwrap_or("");

            let target = format!("/system/{}", name_str);
            if !Path::new(&target).exists() {
                continue;
            }

            let mut mount_success = false;

            if decoy_enabled && !decoy_folder.is_empty() && Path::new(decoy_folder).exists() {
                let decoy_path = format!("{}/system/{}", decoy_folder, name_str);
                let _ = fs::create_dir_all(&decoy_path);
                let lowerdir = format!("{}:{}/{}:{}", decoy_path, pwd_str, name_str, target);
                if mount::mount_overlay(&lowerdir, &target, fs_alias, device_name).is_ok() {
                    mount_success = true;
                }
            } else {
                let lowerdir = format!("{}:{}", pwd_str, target);
                if mount::mount_overlay(&lowerdir, &target, fs_alias, device_name).is_ok() {
                    mount_success = true;
                }
            }

            if mount_success {
                let log_entry = format!("/system/{}", name_str);
                log_mount_list(&log_entry);
            }
        }
    }

    Ok(())
}

fn controlled_depth(
    cwd: &str,
    partition: &str,
    prefix: &str,
    fs_alias: &str,
    device_name: &str,
    decoy_enabled: bool,
    decoy_folder: &str,
) -> anyhow::Result<()> {
    let partition_path = format!("{}/{}", cwd, partition);

    if !Path::new(&partition_path).exists() {
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(&partition_path) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }

            let name = entry.file_name();
            let name_str = name.to_str().unwrap_or("");
            let dir_path = entry.path().to_str().unwrap_or("").to_string();

            let target = format!("{}{}/{}", prefix, partition, name_str);
            if !Path::new(&target).exists() {
                continue;
            }

            let mut mount_success = false;

            if decoy_enabled && !decoy_folder.is_empty() && Path::new(decoy_folder).exists() {
                let decoy_path = format!("{}{}/{}", decoy_folder, prefix, partition);
                let _ = fs::create_dir_all(&decoy_path);
                let lowerdir = format!(
                    "{}:{}/{}:{}",
                    decoy_path, dir_path, name_str, target
                );
                if mount::mount_overlay(&lowerdir, &target, fs_alias, device_name).is_ok() {
                    mount_success = true;
                }
            } else {
                let lowerdir = format!("{}:{}", dir_path, target);
                if mount::mount_overlay(&lowerdir, &target, fs_alias, device_name).is_ok() {
                    mount_success = true;
                }
            }

            if mount_success {
                log_mount_list(&target);
            }
        }
    }

    Ok(())
}

fn log_mount_list(path: &str) {
    let log_path = format!("{}/mountify_mount_list", util::LOG_FOLDER);
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|f| {
            use std::io::Write;
            writeln!(&f, "{}", path)
        });
}
