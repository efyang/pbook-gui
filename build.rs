use std::env;
use std::fs::copy;
use std::io::prelude::*;
use std::fs::File;
use std::process::Command;

fn main() {
    // update git submodule
    Command::new("git")
        .arg("submodule")
        .arg("init")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    let fsep;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    if cfg!(windows) {
        fsep = "\\";
        let outdir = env::var("OUT_DIR").unwrap();
        let depdir;
        if cfg!(target_pointer_width = "64") {
            // 64 bit
            depdir = format!("{}{}", manifest_dir, "\\windows-deps\\x86_64");
        } else {
            // 32 bit
            depdir = format!("{}{}", manifest_dir, "\\windows-deps\\x86");
        }
        println!("cargo:rustc-link-search={}", depdir);
        println!("{}", outdir);
        copy(format!("{}{}",
                     manifest_dir,
                     "\\windows-deps\\pbook-gui.exe.manifest"),
             format!("{}{}", outdir, "\\..\\..\\..\\pbook-gui.exe.manifest"))
            .expect("Failed to copy manifest");
        copy(format!("{}{}", depdir, "\\iup.dll"),
             format!("{}{}", outdir, "\\..\\..\\..\\iup.dll"))
            .expect("Failed to copy iup.dll");
    } else {
        fsep = "/";
    }
    // store pbooks.md file to include.rs
    let src_dir = format!("{}{}{}", manifest_dir, fsep, "src");
    let mut include_file = File::create(format!("{}{}{}", src_dir, fsep, "include.rs"))
                               .expect("Failed to create \"include.rs\" file");
    let pbook_data_path = format!("{}{}{}{}{}",
                                  manifest_dir,
                                  fsep,
                                  "free-programming-books",
                                  fsep,
                                  "free-programming-books.md");
    let include_str = format!("pub const RAW_DATA: &'static str = include_str!(\"{}\");",
                              pbook_data_path);
    include_file.write_all(include_str.as_bytes())
                .expect("Failed to write include_str to include.rs");
}
