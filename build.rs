use std::env;
use std::fs::copy;
use std::path::Path;

fn main() {
    let fsep;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    if cfg!(windows) {
        fsep = "\\";
        let outdir = env::var("OUT_DIR").unwrap();
        let depdir; 
        if cfg!(target_pointer_width = "64") {
            //64 bit
            depdir = format!("{}{}", manifest_dir, "\\windows-deps\\x86_64");
        } else {
            //32 bit
            depdir = format!("{}{}", manifest_dir, "\\windows-deps\\x86");
        }
        println!("cargo:rustc-link-search={}", depdir);
        println!("{}", outdir);
        copy(format!("{}{}", manifest_dir, "\\windows-deps\\pbook-gui.exe.manifest"), format!("{}{}", outdir, "\\..\\..\\..\\pbook-gui.exe.manifest")).expect("Failed to copy manifest");
        copy(format!("{}{}", depdir, "\\iup.dll"), format!("{}{}", outdir, "\\..\\..\\..\\iup.dll")).expect("Failed to copy iup.dll");
    } else {
        fsep = "/";
    }
    
    let pbook_raw = format!("{root}{s}free-programming-books{s}free-programming-books.md", root = manifest_dir, s = fsep);
}
