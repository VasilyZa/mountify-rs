use crate::util;

pub fn run() -> anyhow::Result<()> {
    println!("[+] mountify");
    println!("[+] extended status");
    println!();

    let diff_path = format!("{}/mount_diff", util::MODDIR);
    if util::file_exists(&diff_path) {
        if let Ok(content) = util::read_file(&diff_path) {
            print!("{}", content);
        }
    } else {
        println!("[!] no logs found!");
    }

    Ok(())
}
