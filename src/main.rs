#![allow(dead_code, unused_imports, unused_attributes)]
#![feature(convert)]
extern crate hyper;
extern crate gtk;
extern crate gdk;
extern crate threadpool;
extern crate num_cpus;

use threadpool::ThreadPool;
use std::env;

mod download;
mod include;
mod gui;
mod parse;

use parse::*;
use include::RAW_DATA;

fn main() {
    // match env::current_exe() {
    //     Ok(exe_path) => println!("Path of this executable is: {}", exe_path.display()),
    //     Err(e) => println!("failed to get current exe path: {}", e),
    // };

    // if the user has a low amount of threads it should be safe to assume that their internet
    // isn't that good and their computer can't handle 4 threads
    let max_cpus = num_cpus::get();
    let max_threads;
    if max_cpus <= 4 {
        max_threads = max_cpus;
    } else {
        max_threads = 4;
    }
    let pool = ThreadPool::new(max_threads);

    let parsed_data = parse(RAW_DATA);
    let downloadthreads_data = parsed_data.clone();

    // start gtk gui
    gui::gui(parsed_data);
}
