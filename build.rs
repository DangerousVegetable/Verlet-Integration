use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    // This tells Cargo to rerun this script if something in /textures/ changes.
    println!("cargo::rerun-if-changed=textures/*");

    let out_dir = std::path::Path::new("./target/release/");
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("textures/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
 