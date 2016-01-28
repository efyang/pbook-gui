use std::process::Command;
use std::env;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;


pub fn main() {
    // update git submodule
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    let manifest_dir_str = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir_str);
    let src_dir = manifest_dir.join("src");
    let resource_dir = manifest_dir.join("resources");
    let mut include_file = File::create(src_dir.join("include.rs"))
        .expect("Failed to create \"include.rs\" file");
    let pbook_data_path = resource_dir.join("free-programming-books")
        .join("free-programming-books.md");
    let include_str = format!("pub const RAW_DATA: &'static str = include_str!(\"{}\");",
    double_slashes(pbook_data_path.to_str().unwrap()));
    include_file.write_all(include_str.as_bytes())
        .expect("Failed to write include_str to include.rs");

}

#[cfg(windows)]
fn double_slashes(path: &str) -> String {
    path.replace("\\\\\\\\", "\\")
        .replace("\\\\\\", "\\")
        .replace("\\\\", "\\")
        .replace("\\", "\\\\")
}
#[cfg(not(windows))]
fn double_slashes(path: &str) -> String {
    path.to_string()
}
