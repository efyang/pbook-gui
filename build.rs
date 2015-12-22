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

// build script should be run twice to package

#[cfg(windows)]
const FSEP: &'static str = "\\";

#[cfg(not(windows))]
const FSEP: &'static str = "/";

// keccak-224 hash
#[cfg(target_pointer_width = "32")]
const LIB_CHECKSUM: &'static str = "a8cbe4f2f60d2f2013babab09a467129d846e8d833895ae5711845e4";

#[cfg(target_pointer_width = "64")]
const LIB_CHECKSUM: &'static str = "da79d49ed03ae6695d9cf22f259937d839ae6b7971b3c184352e2fcc";

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
        let deps_dir = double_slashes(deps.to_str().unwrap());
        let dlout = deps.join(file_name);
        match create_dir_all(deps.clone()) {
            Ok(_) => {}
            Err(_) => panic!("Failed to make dir \"deps\""),
        }
        let zpath = format!(".{s}{s}7z{s}{s}{arch}{s}{s}7za.exe", s = FSEP, arch = arch);
        // lib file doesnt exist or checksum is incorrect -> redownload
        if !(dlout.exists() && &file_sha3_hash(&dlout).unwrap_or("".to_string()) == LIB_CHECKSUM) {
            let mut outfile = BufWriter::new(File::create(dlout.clone())
                                                 .expect("Failed to make gtk.7z file"));
            let stream = try_until_stream(download_link, 5);
            for byte in stream.bytes() {
                outfile.write(&[byte.unwrap()]).expect(&format!("Failed to write to {}", file_name));
            }
            outfile.flush().expect(&format!("Failed to flush to {}", file_name));
            drop(outfile);
            // unzip the downloaded libraries
            println!("cargo:rerun-if-changed={}", deps_dir);
            Command::new(zpath.clone())
                .arg("x")
                .arg(dlout.to_str().unwrap())
                .arg("-o.\\\\deps")
                .arg("-y")
                .output()
                .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
        }
                // let cargo search the unzipped dir
        println!("cargo:rustc-link-search=native={}",
                 format!("{deps}{s}{s}lib{b}", deps = deps_dir, s = FSEP, b = bitsize));

        // create the dist dir
        let out_dir = double_slashes(&out_dir);
        let out_dir_path = Path::new(&out_dir);
        println!("cargo:rerun-if-changed={}",
                 (*out_dir_path).to_str().unwrap());
        let dist_dir = out_dir_path.join("programming-book-downloader");
        let bin_dir = dist_dir.join("bin");
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        copy_dir(&deps.join(format!("lib{}", bitsize)), &bin_dir)
            .expect("Failed to copy all deps to bin dir");
        let launcher_path = out_dir_path.join("pbook-launcher.exe");
        let main_path = out_dir_path.join("pbook-gui.exe");
        copy(gtk_css_path.clone(), out_dir_path.join("gtk.css")).expect("Failed to copy gtk.css");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");

        add_themes(&Path::new(&manifest_dir), &out_dir_path, &dist_dir);

        if launcher_path.exists() && main_path.exists() {
            copy(launcher_path, dist_dir.join("pbook-launcher.exe"))
                .expect("Failed to copy launcher");
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
        println!("cargo:rerun-if-changed={}",
                 (*out_dir_path).to_str().unwrap());
        let dist_dir = out_dir_path.join("programming-book-downloader");
        let bin_dir = dist_dir.join("bin");
        create_dir_all(bin_dir.clone()).expect("Failed to create bin dir");
        let launcher_path = out_dir_path.join("pbook-launcher");
        let main_path = out_dir_path.join("pbook-gui");
        copy(gtk_css_path.clone(), out_dir_path.join("gtk.css")).expect("Failed to copy gtk.css");
        copy(gtk_css_path, dist_dir.join("gtk.css")).expect("Failed to copy gtk.css");

        add_themes(&Path::new(&manifest_dir), &out_dir_path, &dist_dir);

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

fn add_themes(manifest_dir: &Path, out_dir: &Path, dist_dir: &Path) {
    delete_if_exists(&out_dir.join("themes"));
    delete_if_exists(&dist_dir.join("themes"));
    if cfg!(feature = "all-themes") || cfg!(feature = "arc") || cfg!(feature = "arc-darker") ||
       cfg!(feature = "arc-dark") ||
       cfg!(feature = "arc-solid") || cfg!(feature = "arc-darker-solid") ||
       cfg!(feature = "arc-dark-solid") || cfg!(feature = "iris-light") ||
       cfg!(feature = "iris-dark") {
        // theme should be active -> add it in to gtk.css in dist instances
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
            copy_themes(&manifest_dir.join("themes"),
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
            copy_themes(&manifest_dir.join("themes"), &theme_dir, dist_dir, &themes);
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
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained.css")], &theme_out);
            rename(theme_out.join("gtk-contained.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-darker" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
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
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-dark.css")], &theme_out);
            rename(theme_out.join("gtk-contained-dark.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"arc-solid" {
            theme_root = theme_root_dir.join("Arc-theme")
                                       .join("common")
                                       .join("gtk-3.0")
                                       .join("3.18");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
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
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
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
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
            remove_all_css_besides(&vec![&theme_out.join("gtk-contained-solid-dark.css")],
                                   &theme_out);
            rename(theme_out.join("gtk-contained-solid-dark.css"),
                   theme_out.join("gtk.css"))
                .expect("Failed to rename");
        } else if theme_name == &"iris-light" {
            theme_root = theme_root_dir.join("iris-light")
                                       .join("gtk-3.0");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
        } else if theme_name == &"iris-dark" {
            theme_root = theme_root_dir.join("iris")
                                       .join("gtk-3.0");
            copy_dir(&theme_root, &theme_out).expect("Failed to copy dir");
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

fn double_slashes(path: &str) -> String {
    path.replace("\\", "\\\\")
}
