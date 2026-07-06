use std::fs;
use std::os::android::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::selinux;
use crate::util;

pub fn create_whiteout(path: &str, whitout_dir: &str) -> anyhow::Result<()> {
    let full_path = format!("{}{}", whitout_dir, path);

    if let Some(parent) = Path::new(&full_path).parent() {
        fs::create_dir_all(parent)?;
        let mode = fs::metadata("/system")?.st_mode();
        let _ = fs::set_permissions(parent, fs::Permissions::from_mode(mode));
    }

    let cpath = std::ffi::CString::new(full_path.as_str())
        .map_err(|_| anyhow::anyhow!("invalid path"))?;

    let ret = unsafe {
        libc::mknod(
            cpath.as_ptr(),
            libc::S_IFCHR | 0o644,
            libc::makedev(0, 0),
        )
    };

    if ret != 0 {
        let errno = std::io::Error::last_os_error().to_string();
        return Err(anyhow::anyhow!("mknod({}): {}", full_path, errno));
    }

    if let Ok(ctx) = selinux::getxattr("/system", "security.selinux") {
        let _ = selinux::setxattr(&full_path, "security.selinux", &ctx);
    }

    selinux::set_whiteout(&full_path)?;

    Ok(())
}

pub fn generate_whiteouts(whiteout_list: &str, output_dir: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(whiteout_list)?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let normalized = if line.starts_with('/') && !line.starts_with("/system/") {
            format!("/system{}", line)
        } else if !line.starts_with("/system/") {
            util::dmesg(&format!("Invalid whiteout path, skipping: {}", line));
            continue;
        } else {
            line.to_string()
        };

        create_whiteout(&normalized, output_dir)?;
    }

    create_target_symlinks(output_dir)?;

    Ok(())
}

fn create_target_symlinks(output_dir: &str) -> anyhow::Result<()> {
    for target in util::TARGETS {
        let dir = format!("/{}", target);
        let system_dir = format!("{}/system/{}", output_dir, target);
        let symlink_dir = format!("{}/{}", output_dir, target);

        if !Path::new(&dir).exists() || Path::new(&dir).is_symlink() {
            continue;
        }
        if !Path::new(&system_dir).exists() {
            continue;
        }

        let _ = fs::remove_file(&symlink_dir);
        std::os::unix::fs::symlink(format!("./system/{}", target), &symlink_dir)?;
    }
    Ok(())
}
