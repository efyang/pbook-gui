extern crate hyper;

use std::io::prelude::*;
use std::io::BufWriter;
use std::io;
use std::fs::{create_dir_all, File, copy, read_dir, remove_file};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use hyper::client::Client;
use hyper::client::response::Response;

// build script should be run twice to package

#[cfg(windows)]
const FSEP: &'static str = "\\";

#[cfg(not(windows))]
const FSEP: &'static str = "/";

pub fn main() {
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
    // store pbooks.md file to include.rs
    let manifest_dir = double_slashes(&env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = double_slashes(&format!("{}{}{}", manifest_dir, FSEP, "src"));
    let gtk_css_path = Path::new(&src_dir).join("gtk.css");
    let mut include_file = File::create(format!("{}{}{}", src_dir, FSEP, "include.rs"))
                               .expect("Failed to create \"include.rs\" file");
    let pbook_data_path = format!("{}{}{}{}{}{}{}",
                                  manifest_dir,
                                  FSEP,
                                  FSEP,
                                  "free-programming-books",
                                  FSEP,
                                  FSEP,
                                  "free-programming-books.md");
    let include_str = format!("pub const RAW_DATA: &'static str = include_str!(\"{}\");",
                              pbook_data_path);
    include_file.write_all(include_str.as_bytes())
                .expect("Failed to write include_str to include.rs");
    let out_dir = env::var("OUT_DIR").unwrap() + &format!("{s}..{s}..{s}..", s = FSEP);
    if cfg!(windows) {
        let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let download_link;
        let file_name;
        let arch;
        let bitsize;
        if cfg!(target_pointer_width = "32") {
            // 32 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib32.7z";
            file_name = "gtk32.7z";
            arch = "x86";
            bitsize = "32";
        } else {
            // 64 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib64.7z";
            file_name = "gtk64.7z";
            arch = "x64";
            bitsize = "64";
        }
        let deps = Path::new(&root_dir).join("deps");
        let dlout = deps.join(file_name);
        match create_dir_all(deps.clone()) {
            Ok(_) => {},
            Err(_) => panic!("Failed to make dir \"deps\""),
        }
        let mut outfile = BufWriter::new(File::create(dlout.clone()).expect("Failed to make gtk.7z file"));
        let stream = try_until_stream(download_link, 5);
        for byte in stream.bytes() {
            outfile.write(&[byte.unwrap()]).expect(&format!("Failed to write to {}", file_name));
        }
        outfile.flush().expect(&format!("Failed to flush to {}", file_name));
        drop(outfile);
        // unzip the downloaded libraries
        let zpath = format!(".{s}{s}7z{s}{s}{arch}{s}{s}7za.exe", s = FSEP, arch = arch);
        let deps_dir = double_slashes(deps.to_str().unwrap());
        println!("cargo:rerun-if-changed={}", deps_dir);
        Command::new(zpath.clone())
            .arg("x")
            .arg(dlout.to_str().unwrap())
            .arg("-o.\\\\deps")
            .arg("-y")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        // let cargo search the unzipped dir
        println!("cargo:rustc-link-search=native={}", format!("{deps}{s}{s}lib{b}",
                    deps = deps_dir,
                    s = FSEP,
                    b = bitsize));

        // create the dist dir
        let out_dir = double_slashes(&out_dir);
        let out_dir_path = Path::new(&out_dir);
        println!("cargo:rerun-if-changed={}", (*out_dir_path).to_str().unwrap());
        let dist_dir = out_dir_path.join("programming-book-downloader");
        let bin_dir = dist_dir.join("bin");
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        copy_dir(&deps.join(format!("lib{}", bitsize)), &bin_dir).expect("Failed to copy all deps to bin dir");
        let launcher_path = out_dir_path.join("pbook-launcher.exe");
        let main_path = out_dir_path.join("pbook-gui.exe");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        if launcher_path.exists() && main_path.exists() {
            copy(launcher_path, dist_dir.join("pbook-launcher.exe")).expect("Failed to copy launcher");
            copy(main_path, bin_dir.join("pbook-gui.exe")).expect("Failed to copy main executable");
            // zip it all up
            let archive_path = out_dir_path.join("programming-book-downloader.zip");
            delete_if_exists(&archive_path);
            Command::new(zpath)
                .arg("a")
                .arg(archive_path.to_str().unwrap())
                .arg(dist_dir.to_str().unwrap())
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
    } else {
        // create the dist dir
        let out_dir_path = Path::new(&out_dir);
        println!("cargo:rerun-if-changed={}", (*out_dir_path).to_str().unwrap());
        let dist_dir = out_dir_path.join("programming-book-downloader");
        let bin_dir = dist_dir.join("bin");
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        let launcher_path = out_dir_path.join("pbook-launcher");
        let main_path = out_dir_path.join("pbook-gui");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        if launcher_path.exists() && main_path.exists() {
            copy(launcher_path, dist_dir.join("pbook-launcher")).expect("Failed to copy launcher");
            copy(main_path, bin_dir.join("pbook-gui")).expect("Failed to copy main executable");
            // tarball everything
            let archive_path = out_dir_path.join("programming-book-downloader.tar.xz");
            delete_if_exists(&archive_path);
            Command::new("tar")
                .arg("cfJ")
                .arg(archive_path.to_str().unwrap())
                .arg("-C")
                .arg(out_dir_path.to_str().unwrap())
                .arg("programming-book-downloader")
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
    }
}

fn delete_if_exists(file: &Path) {
    if (*file).exists() {
        if (*file).metadata().unwrap().is_file() {
            remove_file(file.clone()).expect(&format!("Failed to delete file \"{}\"", file.to_str().unwrap()));
        }
    }
}

// taken from https://github.com/aochagavia/rocket/blob/master/build.rs
fn copy_dir(source_path: &PathBuf, target_path: &PathBuf) -> io::Result<()> {
    match read_dir(source_path) {
        Ok(entry_iter) => {
            try!(create_dir_all(target_path));
            for entry in entry_iter {
                let entry = try!(entry);
                let source_path = entry.path();
                let target_path = target_path.join(entry.file_name());
                try!(copy_dir(&source_path, &target_path));
            }
        }
        Err(_) => {
            try!(copy(&source_path, &target_path));
        }
    }
    Ok(())
}

fn try_until_stream(link: &str, maxtimes: usize) -> Response {
    let client = Client::new();
    let mut stream = None;
    for _ in 0..maxtimes {
        match client.get(link).send() {
            Ok(res) => {
                stream = Some(res);
                break;
            },
            Err(_) => {},
        }
    }

    match stream {
        Some(r) => r,
        None => panic!("Failed to connect"),
    }
}

fn double_slashes(path: &str) -> String {
    path.replace("\\", "\\\\")
}
