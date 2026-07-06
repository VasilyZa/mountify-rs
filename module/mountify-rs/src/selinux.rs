use std::ffi::CString;
use std::path::Path;

pub(crate) fn setxattr(path: &str, name: &str, value: &[u8]) -> anyhow::Result<()> {
    let cpath = CString::new(path)
        .map_err(|_| anyhow::anyhow!("invalid path: {}", path))?;
    let cname = CString::new(name)
        .map_err(|_| anyhow::anyhow!("invalid xattr name: {}", name))?;
    let ret = unsafe {
        libc::setxattr(
            cpath.as_ptr(),
            cname.as_ptr(),
            value.as_ptr() as *const libc::c_void,
            value.len(),
            0,
        )
    };
    if ret != 0 {
        return Err(anyhow::anyhow!("setxattr({}, {}): {}", path, name, errno_str()));
    }
    Ok(())
}

pub(crate) fn getxattr(path: &str, name: &str) -> anyhow::Result<Vec<u8>> {
    let cpath = CString::new(path).map_err(|_| anyhow::anyhow!("invalid path: {}", path))?;
    let cname = CString::new(name).map_err(|_| anyhow::anyhow!("invalid xattr name: {}", name))?;
    let mut buf = vec![0u8; 4096];
    let ret = unsafe {
        libc::getxattr(
            cpath.as_ptr(),
            cname.as_ptr(),
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
        )
    };
    if ret < 0 {
        return Err(anyhow::anyhow!("getxattr({}, {}): {}", path, name, errno_str()));
    }
    buf.truncate(ret as usize);
    Ok(buf)
}

fn errno_str() -> String {
    std::io::Error::last_os_error().to_string()
}

pub fn chcon_reference(target: &str, reference: &str) -> anyhow::Result<()> {
    let ctx = getxattr(reference, "security.selinux")?;
    setxattr(target, "security.selinux", &ctx)?;
    Ok(())
}

pub fn mirror_selinux_contexts(base_dir: &str, dest_dir: &str) -> anyhow::Result<()> {
    let base = Path::new(base_dir);
    let dest = Path::new(dest_dir);

    if !base.exists() || !dest.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(base).follow_links(true) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(base)?;
        let dest_path = dest.join(relative);

        if dest_path.exists() {
            chcon_reference(
                dest_path.to_str().unwrap_or_default(),
                entry.path().to_str().unwrap_or_default(),
            )?;
        }
    }
    Ok(())
}

pub fn set_opaque(dir: &str) -> anyhow::Result<()> {
    setxattr(dir, "trusted.overlay.opaque", b"y")
}

pub fn set_whiteout(path: &str) -> anyhow::Result<()> {
    setxattr(path, "trusted.overlay.whiteout", b"y")
}

pub fn has_opaque(dir: &str) -> bool {
    getxattr(dir, "trusted.overlay.opaque").map(|v| v == b"y").unwrap_or(false)
}

pub fn mirror_opaque_dirs(base_dir: &str, dest_dir: &str) -> anyhow::Result<()> {
    let base = Path::new(base_dir);
    let dest = Path::new(dest_dir);

    for entry in walkdir::WalkDir::new(base).follow_links(true) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let relative = entry.path().strip_prefix(base)?;
        let dest_path = dest.join(relative);

        if dest_path.exists() && has_opaque(entry.path().to_str().unwrap_or_default()) {
            set_opaque(dest_path.to_str().unwrap_or_default())?;
        }
    }
    Ok(())
}
