use std::fs;
use std::path::Path;

use crate::util;

pub fn run() -> anyhow::Result<()> {
    // remove skip_mount on modules we skip_mounted
    let skipped_path = format!("{}/skipped_modules", util::PERSISTENT_DIR);
    if let Ok(content) = fs::read_to_string(&skipped_path) {
        for module in content.lines() {
            let module = module.trim();
            if !module.is_empty() {
                let skip_path = format!("/data/adb/modules/{}/skip_mount", module);
                let _ = fs::remove_file(&skip_path);
            }
        }
    }

    // remove flags created by mountify webui
    let flags = ["/data/adb/.litemode_enable"];
    for flag in &flags {
        if Path::new(flag).exists() {
            if let Ok(content) = fs::read_to_string(flag) {
                if content.contains("mountify") {
                    let _ = fs::remove_file(flag);
                }
            }
        }
    }

    // delete config directory
    if Path::new(util::PERSISTENT_DIR).exists() {
        fs::remove_dir_all(util::PERSISTENT_DIR)?;
    }

    Ok(())
}
