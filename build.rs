extern crate hyper;

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::{create_dir_all, File};
use std::env;
use std::path::Path;
use hyper::client::Client;

fn main() {
    if cfg!(windows) {
        let out_dir = env::var("OUT_DIR").unwrap();
        let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let download_link;
        if cfg!(target_pointer_width = "32") {
            // 32 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib32.7z";
        } else {
            // 64 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib64.7z"
        }
        let client = Client::new();
        let deps = Path::new(&root_dir).join("deps");
        match create_dir_all(deps.clone()) {
            Ok(_) => {},
            Err(_) => panic!("Failed to make dir \"deps\""),
        }
        let mut outfile = File::create(deps.join("gtk.7z")).unwrap();
        let res = client.get(download_link).send().unwrap();

    }
}

fn try_until_stream(link: &str) {

}
