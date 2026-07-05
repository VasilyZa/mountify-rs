use crate::util;
use crate::whiteout;

pub fn run(list: &str, output: &str) -> anyhow::Result<()> {
    println!("[+] mountify's whiteout generator");

    let list_path = if util::file_exists(list) {
        list.to_string()
    } else {
        let fallback = format!("{}/whiteouts.txt", util::PERSISTENT_DIR);
        if util::file_exists(&fallback) {
            fallback
        } else {
            return Err(anyhow::anyhow!("whiteout list not found"));
        }
    };

    // create output directory
    util::ensure_dir(output)?;

    // set selinux context
    let _ = std::process::Command::new("busybox")
        .arg("chcon")
        .arg("--reference")
        .arg("/system")
        .arg(output)
        .status();

    // generate whiteouts
    whiteout::generate_whiteouts(&list_path, output)?;

    // copy whiteout module resources
    let mod_whiteout = format!("{}/whiteout", util::MODDIR);
    let output_mod_prop = format!("{}/module.prop", output);
    let output_action_sh = format!("{}/action.sh", output);

    let src_mod_prop = format!("{}/module.prop", mod_whiteout);
    let src_action_sh = format!("{}/action.sh", mod_whiteout);

    if util::file_exists(&src_mod_prop) {
        let content = util::read_file(&src_mod_prop)?;
        util::write_file(&output_mod_prop, &content)?;
    }
    if util::file_exists(&src_action_sh) {
        let content = util::read_file(&src_action_sh)?;
        util::write_file(&output_action_sh, &content)?;
    }

    Ok(())
}
