use std::fs;
use std::path::Path;
use std::process::Command;

use crate::util;

pub fn run(modid: Option<String>, modpath: Option<String>) -> anyhow::Result<()> {
    let modid = modid.unwrap_or_default();
    let modpath = modpath.unwrap_or_default();

    // these environment variables are set by the parent script
    // we just handle the hot install logic here

    let hot_install_enabled = std::env::var("MODULE_HOT_INSTALL_REQUEST")
        .map(|v| v == "true")
        .unwrap_or(false);

    if !hot_install_enabled || modid.is_empty() || modpath.is_empty() {
        return Ok(());
    }

    let moddir_internal = format!("/data/adb/modules/{}", modid);
    let modpath_internal = format!("/data/adb/modules_update/{}", modid);

    if !Path::new(&moddir_internal).exists() || !Path::new(&modpath_internal).exists() {
        return Ok(());
    }

    // hot install
    let _ = Command::new("busybox")
        .arg("rm")
        .arg("-rf")
        .arg(&moddir_internal)
        .status();

    let _ = Command::new("busybox")
        .arg("mv")
        .arg(&modpath_internal)
        .arg(&moddir_internal)
        .status();

    // run script if requested
    let hot_run_script = std::env::var("MODULE_HOT_RUN_SCRIPT").unwrap_or_default();
    if !hot_run_script.is_empty() {
        let script_path = format!("{}/{}", moddir_internal, hot_run_script);
        if Path::new(&script_path).exists() {
            let _ = Command::new("sh")
                .arg(&script_path)
                .status();
        }
    }

    // satisfy ensure_file_exists
    util::ensure_dir(&modpath_internal)?;
    let modprop_src = format!("{}/module.prop", moddir_internal);
    let modprop_dst = format!("{}/module.prop", modpath_internal);
    if Path::new(&modprop_src).exists() {
        let content = fs::read_to_string(&modprop_src)?;
        fs::write(&modprop_dst, &content)?;
    }

    // fork cleanup in background
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let _ = fs::remove_dir_all(format!("{}/update", moddir_internal));
        let _ = fs::remove_dir_all(&modpath_internal);
    });

    println!("- Module hot install requested!");
    println!("- Refresh module page after installation!");
    println!("- No need to reboot!");

    Ok(())
}
