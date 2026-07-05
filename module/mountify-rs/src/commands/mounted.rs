use std::fs;

use crate::util;

pub fn run() -> anyhow::Result<()> {
    let modules_path = format!("{}/modules", util::LOG_FOLDER);
    let mounts_path = format!("{}/mountify_mount_list", util::LOG_FOLDER);

    if !util::file_exists(&modules_path) {
        println!("[!] no mounted modules found");
        return Ok(());
    }

    let content = fs::read_to_string(&modules_path)?;
    let modules: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

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
