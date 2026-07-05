use std::fs;

fn errno_str() -> String {
    std::io::Error::last_os_error().to_string()
}

fn init_module(image: &str, params: &str) -> anyhow::Result<()> {
    let image_data = fs::read(image)?;
    let cparams = std::ffi::CString::new(params)
        .map_err(|_| anyhow::anyhow!("invalid module params"))?;

    let ret = unsafe {
        libc::syscall(
            libc::SYS_init_module,
            image_data.as_ptr() as *const libc::c_void,
            image_data.len(),
            cparams.as_ptr(),
        )
    };

    if ret != 0 {
        return Err(anyhow::anyhow!("init_module({}): {}", image, errno_str()));
    }
    Ok(())
}

fn finit_module(fd: std::os::unix::io::RawFd, params: &str, flags: libc::c_int) -> anyhow::Result<()> {
    let cparams = std::ffi::CString::new(params)
        .map_err(|_| anyhow::anyhow!("invalid module params"))?;

    let ret = unsafe {
        libc::syscall(libc::SYS_finit_module, fd, cparams.as_ptr(), flags)
    };

    if ret != 0 {
        return Err(anyhow::anyhow!("finit_module: {}", errno_str()));
    }
    Ok(())
}

pub fn insmod(image: &str, params: &str) -> anyhow::Result<()> {
    match std::fs::File::open(image) {
        Ok(file) => {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            match finit_module(fd, params, 0) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    crate::util::dmesg(&format!("finit_module failed for {}, falling back: {}", image, e));
                }
            }
        }
        Err(e) => {
            crate::util::dmesg(&format!("cannot open {}: {}", image, e));
        }
    }

    init_module(image, params)
}

pub fn load_nuke_lkm(
    lkm_path: &str,
    mount_point: &str,
) -> anyhow::Result<()> {
    let kptr_restrict = fs::read_to_string("/proc/sys/kernel/kptr_restrict")
        .unwrap_or_default();

    let _ = fs::write("/proc/sys/kernel/kptr_restrict", "1\n");

    let kallsyms = fs::read_to_string("/proc/kallsyms").unwrap_or_default();
    let symaddr = kallsyms
        .lines()
        .find(|l| l.contains(" ext4_unregister_sysfs$"))
        .and_then(|l| l.split_whitespace().next())
        .map(|addr| format!("0x{}", addr))
        .unwrap_or_default();

    let params = format!("mount_point={} symaddr={}", mount_point, symaddr);

    let result = insmod(lkm_path, &params);

    let _ = fs::write("/proc/sys/kernel/kptr_restrict", kptr_restrict.trim());

    result
}

pub fn ksud_nuke_ext4(mount_point: &str) -> anyhow::Result<()> {
    let exit = std::process::Command::new("/data/adb/ksud")
        .arg("kernel")
        .arg("nuke-ext4-sysfs")
        .arg(mount_point)
        .status()
        .map_err(|e| anyhow::anyhow!("ksud failed: {}", e))?;

    if exit.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("ksud nuke-ext4-sysfs returned non-zero"))
    }
}
