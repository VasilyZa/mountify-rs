use std::fs;

use crate::anti_bootloop;
use crate::util;

pub fn run() -> anyhow::Result<()> {
    // reset bootcount
    anti_bootloop::BootCount::reset()?;

    // remove single instance lock
    if util::file_exists(util::MOUNTIFY_LOCK) {
        util::dmesg("mountify/boot-completed: lifting single instance lock");
        fs::remove_file(util::MOUNTIFY_LOCK)?;
    }

    // clean log folder
    if util::file_exists(util::LOG_FOLDER) {
        fs::remove_dir_all(util::LOG_FOLDER)?;
    }

    Ok(())
}
