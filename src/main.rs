#![allow(dead_code, unused_imports, unused_attributes)]
extern crate hyper;
#[macro_use]
extern crate gtk;

mod download;
mod include;
mod gui;
mod parse;

use include::RAW_DATA;

use std::env;
use parse::*;

fn main() {
    match env::current_exe() {
        Ok(exe_path) => println!("Path of this executable is: {}", exe_path.display()),
        Err(e) => println!("failed to get current exe path: {}", e),
    };

    let parsed = parse(RAW_DATA);

    // start gtk gui
    gui::gui(parsed);
}
