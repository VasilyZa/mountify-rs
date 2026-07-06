use crate::util;

pub const BOOTCOUNT_FILE: &str = "/data/adb/modules/mountify-rs/count.sh";

#[derive(Debug)]
pub struct BootCount {
    pub count: u32,
    pub should_disable: bool,
}

impl BootCount {
    pub fn check() -> Self {
        let count = read_bootcount();
        let new_count = count + 1;

        let should_disable = new_count > 1
            && !util::file_exists("/data/adb/mountify/explicit_I_want_a_bootloop");

        if should_disable {
            let _ = disable_module();
        } else {
            let _ = write_bootcount(new_count);
        }

        Self {
            count: new_count,
            should_disable,
        }
    }

    pub fn reset() -> anyhow::Result<()> {
        write_bootcount(0)
    }
}

fn read_bootcount() -> u32 {
    let content = std::fs::read_to_string(BOOTCOUNT_FILE).unwrap_or_default();
    for line in content.lines() {
        if line.starts_with("BOOTCOUNT=") {
            if let Some(val) = line.split('=').nth(1) {
                return val.trim().parse().unwrap_or(0);
            }
        }
    }
    0
}

fn write_bootcount(count: u32) -> anyhow::Result<()> {
    std::fs::write(BOOTCOUNT_FILE, format!("BOOTCOUNT={}\n", count))?;
    Ok(())
}

fn disable_module() -> anyhow::Result<()> {
    let disable_path = "/data/adb/modules/mountify-rs/disable";
    std::fs::write(disable_path, "")?;
    let _ = std::fs::remove_file(BOOTCOUNT_FILE);

    let modprop_path = "/data/adb/modules/mountify-rs/module.prop";
    if let Ok(content) = std::fs::read_to_string(modprop_path) {
        let new_content = content
            .lines()
            .map(|line| {
                if line.starts_with("description=") {
                    "description=防启动循环已触发，模块已禁用。启用后可恢复使用。"
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = std::fs::write(modprop_path, &new_content);
    }

    util::dmesg("anti-bootloop triggered, module disabled");
    Ok(())
}
