use std::ffi::CString;
use std::fs;
use std::path::Path;

use crate::util;

pub const MS_BIND: libc::c_ulong = 4096;
pub const MS_RDONLY: libc::c_ulong = 1;
pub const MS_REMOUNT: libc::c_ulong = 32;
pub const MNT_DETACH: libc::c_int = 2;

fn errno_str() -> String {
    std::io::Error::last_os_error().to_string()
}

fn mount(
    source: Option<&str>,
    target: &str,
    fstype: Option<&str>,
    flags: libc::c_ulong,
    data: Option<&str>,
) -> anyhow::Result<()> {
    let csource = source.map(|s| CString::new(s).unwrap());
    let ctarget = CString::new(target)
        .map_err(|_| anyhow::anyhow!("invalid target: {}", target))?;
    let cfstype = fstype.map(|s| CString::new(s).unwrap());
    let cdata = data.map(|s| CString::new(s).unwrap());

    let ret = unsafe {
        libc::mount(
            csource.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            ctarget.as_ptr(),
            cfstype.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            flags,
            cdata.as_ref().map(|s| s.as_ptr() as *const libc::c_void).unwrap_or(std::ptr::null()),
        )
    };

    if ret != 0 {
        return Err(anyhow::anyhow!(
            "mount({:?}, {}, {:?}, {:#x}, {:?}): {}",
            source, target, fstype, flags, data,
            errno_str()
        ));
    }
    Ok(())
}

fn umount(target: &str, flags: libc::c_int) -> anyhow::Result<()> {
    let ctarget = CString::new(target)
        .map_err(|_| anyhow::anyhow!("invalid umount target: {}", target))?;
    let ret = unsafe { libc::umount2(ctarget.as_ptr(), flags) };
    if ret != 0 {
        return Err(anyhow::anyhow!(
            "umount2({}, {}): {}",
            target, flags,
            errno_str()
        ));
    }
    Ok(())
}

pub fn mount_tmpfs(target: &str) -> anyhow::Result<()> {
    mount(Some("tmpfs"), target, Some("tmpfs"), 0, None)
}

pub fn mount_ext4_loop(image: &str, target: &str, readonly: bool) -> anyhow::Result<()> {
    let flags = if readonly { MS_RDONLY } else { 0 };
    mount(Some(image), target, Some("ext4"), flags, Some("loop,rw,noatime,nodiratime"))
}

pub fn mount_overlay(
    lowerdir: &str,
    target: &str,
    fstype_alias: &str,
    device_name: &str,
) -> anyhow::Result<()> {
    let data = format!("lowerdir={}", lowerdir);
    mount(Some(device_name), target, Some(fstype_alias), 0, Some(&data))
}

pub fn mount_overlay_with_decoy(
    lowerdir: &str,
    target: &str,
    decoy_lowerdir: &str,
    fstype_alias: &str,
    device_name: &str,
) -> anyhow::Result<()> {
    let data = format!("lowerdir={}:{}", decoy_lowerdir, lowerdir);
    mount(Some(device_name), target, Some(fstype_alias), 0, Some(&data))
}

pub fn umount_lazy(target: &str) -> anyhow::Result<()> {
    umount(target, MNT_DETACH)
}

pub fn bind_mount(source: &str, target: &str, readonly: bool) -> anyhow::Result<()> {
    mount(Some(source), target, None, MS_BIND, None)?;
    if readonly {
        mount(Some(source), target, None, MS_BIND | MS_RDONLY | MS_REMOUNT, None)?;
    }
    Ok(())
}

pub fn create_ext4_sparse_image(path: &str, size_mb: u64) -> anyhow::Result<()> {
    let size_bytes = size_mb * 1024 * 1024;
    let f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)?;
    f.set_len(size_bytes)?;
    drop(f);

    let exit = std::process::Command::new("/system/bin/mkfs.ext4")
        .arg("-O")
        .arg("^has_journal")
        .arg(path)
        .status()
        .map_err(|e| anyhow::anyhow!("mkfs.ext4 failed: {}", e))?;

    if !exit.success() {
        return Err(anyhow::anyhow!("mkfs.ext4 returned non-zero exit"));
    }
    Ok(())
}

pub fn stage1_mount(mnt_folder: &str) -> anyhow::Result<()> {
    let real = fs::canonicalize(mnt_folder)
        .map_err(|_| anyhow::anyhow!("cannot canonicalize {}", mnt_folder))?;
    let real_str = real.to_str().unwrap_or(mnt_folder);
    util::dmesg(&format!("stage1: mounting {}", real_str));
    mount_tmpfs(real_str)
}

pub fn stage1_umount(mnt_folder: &str) -> anyhow::Result<()> {
    let real = fs::canonicalize(mnt_folder)?;
    util::dmesg(&format!("stage1: unmounting {}", real.display()));
    umount_lazy(real.to_str().unwrap_or(mnt_folder))
}

pub fn stage2_mount_tmpfs(mnt_folder: &str, fake_name: &str) -> anyhow::Result<()> {
    let target = format!("{}/{}", mnt_folder, fake_name);
    let real = fs::canonicalize(&target)?;
    util::dmesg(&format!("stage2/tmpfs: mounting {}", real.display()));
    mount_tmpfs(real.to_str().unwrap_or(&target))
}

pub fn stage2_mount_ext4(
    mnt_folder: &str,
    fake_name: &str,
    sparse_size: u64,
    is_ksu: bool,
) -> anyhow::Result<()> {
    let image_path = format!("{}/mountify-ext4", mnt_folder);
    let target = format!("{}/{}", mnt_folder, fake_name);

    util::dmesg("stage2/ext4: creating sparse image");
    create_ext4_sparse_image(&image_path, sparse_size)?;

    if is_ksu {
        let ctx = format!("{} \0", "u:object_r:ksu_file:s0");
        let cpath = CString::new(image_path.as_str()).unwrap();
        let cname = CString::new("security.selinux").unwrap();
        unsafe {
            libc::setxattr(
                cpath.as_ptr(),
                cname.as_ptr(),
                ctx.as_ptr() as *const libc::c_void,
                ctx.len() - 1,
                0,
            );
        }
    }

    util::dmesg(&format!("stage2/ext4: mounting {}", &target));
    mount_ext4_loop(&image_path, &target, false)
}

pub fn stage2_remount_ext4(
    mnt_folder: &str,
    fake_name: &str,
    spoof_sparse: bool,
    fake_apex_name: &str,
) -> anyhow::Result<()> {
    let image_path = format!("{}/mountify-ext4", mnt_folder);
    let target = format!("{}/{}", mnt_folder, fake_name);

    util::dmesg(&format!("stage2/ext4: unmounting {}", &target));
    umount_lazy(&target)?;

    if spoof_sparse && Path::new("/apex").exists() && !Path::new(&format!("/apex/{}", fake_apex_name)).exists() {
        let apex_ver = format!("/apex/{}@1", fake_apex_name);
        let apex_dir = format!("/apex/{}", fake_apex_name);
        fs::create_dir_all(&apex_ver)?;
        mount_ext4_loop(&image_path, &apex_ver, true)?;
        fs::create_dir_all(&apex_dir)?;
        bind_mount(&apex_ver, &apex_dir, true)?;

        let _ = fs::remove_dir_all(&target);
        std::os::unix::fs::symlink(&apex_dir, &target)?;
    } else {
        util::dmesg(&format!("stage2/ext4: remounting {} as ro", &target));
        mount_ext4_loop(&image_path, &target, true)?;
    }
    Ok(())
}

pub fn umount_fake_mount(mnt_folder: &str, fake_name: &str) -> anyhow::Result<()> {
    let target = format!("{}/{}", mnt_folder, fake_name);
    util::dmesg(&format!("stage2: unmounting {}", &target));
    umount_lazy(&target)
}
