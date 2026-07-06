use std::fs;

use crate::util;

fn scan_modules() -> Vec<String> {
    let dir = "/data/adb/modules";
    let mut modules = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let id = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if !path.join("system").exists() { continue; }
            if util::file_exists(&format!("{}/disable", path.display())) { continue; }
            if util::file_exists(&format!("{}/remove", path.display())) { continue; }
            if util::file_exists(&format!("{}/skip_mountify", path.display())) { continue; }
            if util::file_exists(&format!("{}/system/etc/hosts", path.display())) { continue; }
            modules.push(id);
        }
    }
    modules
}

pub fn run() -> anyhow::Result<()> {
    let mut modules: Vec<String> = Vec::new();

    // Try tmpfs session log first
    let log_path = format!("{}/modules", util::LOG_FOLDER);
    if util::file_exists(&log_path) {
        if let Ok(content) = fs::read_to_string(&log_path) {
            modules = content.lines().filter(|l| !l.trim().is_empty()).map(|s| s.to_string()).collect();
        }
    }

    // Fallback: real-time scan
    if modules.is_empty() {
        modules = scan_modules();
    }

    let mounts_path = format!("{}/mountify_mount_list", util::LOG_FOLDER);

    if modules.is_empty() {
        println!("[!] no mounted modules found");
        return Ok(());
    }

    println!("[+] mounted modules ({}):", modules.len());
    for module in &modules {
        println!("  - {}", module.trim());
    }

    // Also show mount points if available
    if util::file_exists(&mounts_path) {
        let mounts_content = fs::read_to_string(&mounts_path)?;
        let mounts: Vec<&str> = mounts_content.lines().filter(|l| !l.trim().is_empty()).collect();
        if !mounts.is_empty() {
            println!("\n[+] mount points:");
            for m in &mounts {
                println!("  {}", m.trim());
            }
        }
    }

    Ok(())
}
