extern crate hyper;
extern crate tiny_keccak;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::io::BufWriter;
use std::io;
use std::fs::{create_dir_all, File, copy, read_dir, remove_file, remove_dir_all, rename};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use tiny_keccak::Keccak;
use rustc_serialize::hex::ToHex;
use hyper::client::Client;
use hyper::client::response::Response;

// keccak-224 hash
#[cfg(target_pointer_width = "32")]
const LIB_CHECKSUM: &'static str = "3ce40c5bf0dfa7015c1d49bf2d588cff62bc984d1a94d737c0108f64";

#[cfg(target_pointer_width = "64")]
const LIB_CHECKSUM: &'static str = "921fe50cae83e465e09aa15b1506f6dc2b7e55cc35f550f666b71491";

pub fn main() {
    // update git submodule
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    // store pbooks.md file to include.rs
    let current_exe_path = env::current_exe().unwrap();
    let current_exe_dir = current_exe_path.parent().unwrap();
    
    let manifest_dir = current_exe_dir.join("..").join("..");
    let src_dir = manifest_dir.join("src");
    let gtk_css_path = src_dir.join("gtk.css");
    let resource_dir = manifest_dir.join("resources");
    let build_util_path = manifest_dir.join("build").join("utils");
    let out_dir = current_exe_dir;
    let dist_dir = out_dir.join("programming-book-downloader");
    let bin_dir = dist_dir.join("bin");

    let mut include_file = File::create(src_dir.join("include.rs"))
                               .expect("Failed to create \"include.rs\" file");
    let pbook_data_path = resource_dir.join("free-programming-books")
                                      .join("free-programming-books.md");
    let include_str = format!("pub const RAW_DATA: &'static str = include_str!(\"{}\");",
                              double_slashes(pbook_data_path.to_str().unwrap()));
    include_file.write_all(include_str.as_bytes())
                .expect("Failed to write include_str to include.rs");
    build_launcher(&src_dir, &dist_dir);
    if cfg!(windows) {
        let download_link;
        let lib_name;
        let arch;
        let bitsize;
        if cfg!(target_pointer_width = "32") {
            download_link = "https://github.com/honorabrutroll/pbook-gui-deps/raw/master/lib32.7z";
            lib_name = "gtk32.7z";
            arch = "x86";
            bitsize = "32";
        } else {
            download_link = "https://github.com/honorabrutroll/pbook-gui-deps/raw/master/lib64.7z";
            lib_name = "gtk64.7z";
            arch = "x64";
            bitsize = "64";
        }
        let deps = resource_dir.join("deps");
        let dlout = deps.join(lib_name);
        create_dir_all(deps.clone()).expect("Failed to make dir \"deps\"");
        let zpath = build_util_path.join("7z")
                                   .join(arch)
                                   .join("7za.exe");
        // lib file doesnt exist or checksum is incorrect -> redownload
        if !(dlout.exists() && &file_sha3_hash(&dlout).unwrap_or("".to_string()) == LIB_CHECKSUM) {
            let mut outfile = BufWriter::new(File::create(dlout.clone())
                                                 .expect("Failed to make gtk.7z file"));
            let stream = try_until_stream(download_link, 5);
            for byte in stream.bytes() {
                outfile.write(&[byte.unwrap()])
                       .expect(&format!("Failed to write to {}", lib_name));
            }
            outfile.flush().expect(&format!("Failed to flush to {}", lib_name));
            drop(outfile);
        }
        // unzip the downloaded libraries
        let lib_path = deps.join(format!("lib{}", bitsize));
        if !lib_path.exists() {
            Command::new(zpath.clone())
                .args(&["x", dlout.to_str().unwrap(), &("-o".to_string() + deps.to_str().unwrap()), "-y"])
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
        
        // create the dist dir
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        copy_dir(&deps.join(format!("lib{}", bitsize)), &bin_dir)
            .expect("Failed to copy all deps to bin dir");
        let main_path = out_dir.join("pbook-gui.exe");
        copy(gtk_css_path.clone(), out_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        add_themes(&Path::new(&manifest_dir), &out_dir, &dist_dir);
        if main_path.exists() {
            let new_launcher_path = dist_dir.join("pbook-launcher.exe");
            let new_main_path = bin_dir.join("pbook-gui.exe");
            copy(main_path, &new_main_path).expect("Failed to copy main executable");
            // set icon
            let rcedit_path = manifest_dir.join("build")
                                          .join("utils")
                                          .join("rcedit")
                                          .join("rcedit.exe");
            let icon_path = manifest_dir.join("resources")
                                        .join("icons")
                                        .join("pbook.ico");
            set_icon(&rcedit_path, &new_launcher_path, &icon_path);
            set_icon(&rcedit_path, &new_main_path, &icon_path);
            // zip it all up
            let archive_path = out_dir.join("programming-book-downloader.zip");
            delete_if_exists(&archive_path);
            Command::new(zpath)
                .args(&["a", "-tzip", archive_path.to_str().unwrap(), dist_dir.to_str().unwrap()])
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
    } else {
        // create the dist dir
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        let main_path = out_dir.join("pbook-gui");
        copy(gtk_css_path.clone(), out_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");
        add_themes(&Path::new(&manifest_dir), &out_dir, &dist_dir);
        if main_path.exists() {
            copy(main_path, bin_dir.join("pbook-gui")).expect("Failed to copy main executable");
            // tarball everything
            let archive_path = out_dir.join("programming-book-downloader.tar.xz");
            delete_if_exists(&archive_path);
            Command::new("tar")
                .args(&["cfJ",
                        archive_path.to_str().unwrap(),
                        "-C",
                        out_dir.to_str().unwrap(),
                        "programming-book-downloader"])
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
    }
}

fn build_launcher(src_dir: &Path, dist_dir: &Path) {
    if cfg!(windows) {
        Command::new("g++")
            .arg(src_dir.join("launcher.cpp").to_str().unwrap())
            .arg("-o")
            .arg(dist_dir.join("pbook-launcher.exe").to_str().unwrap())
            .arg("-Os")
            .arg("-mwindows")
            .arg("-static")
            .arg("-static-libstdc++")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
    } else {
        Command::new("g++")
            .arg(src_dir.join("launcher.cpp").to_str().unwrap())
            .arg("-o")
            .arg(dist_dir.join("pbook-launcher").to_str().unwrap())
            .arg("-Os")
            .arg("-static")
            .arg("-static-libstdc++")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
    }
}

fn set_icon(rcedit_path: &Path, exe_path: &Path, ico_path: &Path) {
    Command::new(rcedit_path)
        .arg(exe_path.to_str().unwrap())
        .arg("--set-icon")
        .arg(ico_path.to_str().unwrap())
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
}

fn add_themes(manifest_dir: &Path, out_dir: &Path, dist_dir: &Path) {
    delete_if_exists(&out_dir.join("themes"));
    delete_if_exists(&dist_dir.join("themes"));
    if cfg!(feature = "all-themes") || cfg!(feature = "arc") || cfg!(feature = "arc-darker") ||
       cfg!(feature = "arc-dark") ||
       cfg!(feature = "arc-solid") || cfg!(feature = "arc-darker-solid") ||
       cfg!(feature = "arc-dark-solid") || cfg!(feature = "iris-light") ||
       cfg!(feature = "iris-dark") {
        let theme_dir = out_dir.join("themes");
        create_dir_all(theme_dir.clone()).expect("Failed to create theme dir");
        let theme_file = out_dir.join("theme.txt");
        let mut theme_file_handle = File::create(theme_file).expect("Failed to create theme.txt");
        let theme;
        if cfg!(feature = "all-themes") {
            // copy all themes
            // theme should be active -> write it to theme.txt
            if cfg!(feature = "set-arc-darker") {
                theme = "arc-darker";
            } else if cfg!(feature = "set-arc-dark") {
                theme = "arc-dark";
            } else if cfg!(feature = "set-arc-solid") {
                theme = "arc-solid";
            } else if cfg!(feature = "set-arc-darker-solid") {
                theme = "arc-darker-solid";
            } else if cfg!(feature = "set-arc-dark-solid") {
                theme = "arc-dark-solid";
            } else if cfg!(feature = "set-iris-light") {
                theme = "iris-light";
            } else if cfg!(feature = "set-iris-dark") {
                theme = "iris-dark";
            } else {
                theme = "arc";
            }
            copy_themes(&manifest_dir.join("resources").join("themes"),
                        &theme_dir,
                        dist_dir,
                        &vec!["arc",
                              "arc-darker",
                              "arc-dark",
                              "arc-solid",
                              "arc-darker-solid",
                              "arc-dark-solid",
                              "iris-light",
                              "iris-dark"]);
        } else {
            let mut themes = Vec::new();
            let mut tmp_theme = "arc";
            if cfg!(feature = "arc") {
                themes.push("arc");
            }
            if cfg!(feature = "arc-darker") {
                tmp_theme = "arc-darker";
                themes.push("arc-darker");
            }
            if cfg!(feature = "arc-dark") {
                tmp_theme = "arc-dark";
                themes.push("arc-dark");
            }
            if cfg!(feature = "arc-solid") {
                tmp_theme = "arc-solid";
                themes.push("arc-solid");
            }
            if cfg!(feature = "arc-darker-solid") {
                tmp_theme = "arc-darker-solid";
                themes.push("arc-darker-solid");
            }
            if cfg!(feature = "arc-dark-solid") {
                tmp_theme = "arc-dark-solid";
                themes.push("arc-dark-solid");
            }
            if cfg!(feature = "iris-light") {
                tmp_theme = "iris-light";
                themes.push("iris-light");
            }
            if cfg!(feature = "iris-dark") {
                tmp_theme = "iris-dark";
                themes.push("iris-dark");
            }

            if cfg!(feature = "set-arc-darker") {
                tmp_theme = "arc-darker";
            } else if cfg!(feature = "set-arc-dark") {
                tmp_theme = "arc-dark";
            } else if cfg!(feature = "set-arc-solid") {
                tmp_theme = "arc-solid";
            } else if cfg!(feature = "set-arc-darker-solid") {
                tmp_theme = "arc-darker-solid";
            } else if cfg!(feature = "set-arc-dark-solid") {
                tmp_theme = "arc-dark-solid";
            } else if cfg!(feature = "set-iris-light") {
                tmp_theme = "iris-light";
            } else if cfg!(feature = "set-iris-dark") {
                tmp_theme = "iris-dark";
            }
            theme = tmp_theme;
            copy_themes(&manifest_dir.join("resources").join("themes"),
                        &theme_dir,
                        dist_dir,
                        &themes);
        }
        theme_file_handle.write_all(theme.as_bytes()).expect("Failed to write theme to theme.txt");
        drop(theme_file_handle);
        copy(out_dir.join("theme.txt"), dist_dir.join("theme.txt"))
            .expect("Failed to copy themes.txt");
    }
}

fn file_sha3_hash(path: &Path) -> Result<String, std::io::Error> {
    let mut buf = vec![];
    let mut f;
    match File::open(path) {
        Ok(p) => f = p,
        Err(e) => return Err(e),
    }
    match f.read_to_end(&mut buf) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    return Ok(sha3_224_hash(&buf).to_hex());
}

fn sha3_224_hash(data: &[u8]) -> [u8; 28] {
    let mut sha3 = Keccak::new_sha3_224();
    sha3.update(data);
    let mut res: [u8; 28] = [0; 28];
    sha3.finalize(&mut res);
    res
}

fn copy_themes(theme_root_dir: &Path, theme_out_dir: &Path, dist_dir: &Path, themes: &Vec<&str>) {
    for theme_name in themes.iter() {
        let theme_root;
        let theme_out = theme_out_dir.join(theme_name);
        if theme_name == &"arc" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained.css")], &theme_out);
            rename(theme_out.join("gtk-contained.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-darker" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-darker.css")],
                                   &theme_out);
            rename(theme_out.join("gtk-contained-darker.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-dark" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-dark.css")], &theme_out);
            rename(theme_out.join("gtk-contained-dark.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-solid" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-solid.css")],
                                   &theme_out);
            rename(theme_out.join("gtk-contained-solid.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-darker-solid" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-solid-darker.css")],
                                   &theme_out);
            rename(theme_out.join("gtk-contained-solid-darker.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-dark-solid" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir arc");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-solid-dark.css")],
                                   &theme_out);
            rename(theme_out.join("gtk-contained-solid-dark.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"iris-light" {
            theme_root = theme_root_dir.join("iris-light")
                                       .join("gtk-3.0");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir iris-light");
        } else if theme_name == &"iris-dark" {
            theme_root = theme_root_dir.join("iris")
                                       .join("gtk-3.0");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir iris-dark");
        }
    }
    copy_dir(&theme_out_dir.to_path_buf(), &dist_dir.join("themes"))
        .expect("Failed to copy themes");
}
fn remove_all_css_besides(noremove: &Vec<&Path>, dir: &Path) {
    for entry in dir.read_dir().unwrap() {
        let entrypath = entry.expect("Failed while reading dir").path();
        let name = entrypath.to_str().unwrap();
        if &name[name.len() - 4..] == ".css" && ne_to_any(noremove, entrypath.as_path()) {
            delete_if_exists(&entrypath);
        }
    }
}

fn ne_to_any(all: &Vec<&Path>, try: &Path) -> bool {
    for x in all.iter() {
        if x == &try {
            return false;
        }
    }
    return true;
}

fn delete_if_exists(path: &Path) {
    if (*path).exists() {
        if (*path).metadata().unwrap().is_file() {
            remove_file(path.clone())
                .expect(&format!("Failed to delete path \"{}\"", path.to_str().unwrap()));
        } else {
            remove_dir_all(path.clone())
                .expect(&format!("Failed to delete path \"{}\"", path.to_str().unwrap()));
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
            }
            Err(_) => {}
        }
    }
    match stream {
        Some(r) => r,
        None => panic!("Failed to connect"),
    }
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
