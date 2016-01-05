#![allow(dead_code, unused_imports, unused_attributes, unused_variables)]
#![feature(convert)]
extern crate hyper;
extern crate gtk;
extern crate gdk;
extern crate num_cpus;
extern crate threadpool;

use std::env;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

mod data;
mod include;
mod gui;
mod parse;
mod commhandler;
mod download;

use commhandler::*;
use parse::*;
use include::RAW_DATA;

fn main() {
    let threads = num_cpus::get();
    let parsed_data: Vec<Category> = parse(RAW_DATA);
    let downloadthreads_data = parsed_data.clone();

    // initialize the channels between gui and comm handler
    let (gui_update_send, gui_update_recv) = channel::<Vec<Download>>();
    let (gui_cmd_send, gui_cmd_recv) = channel::<(String, Option<u64>)>();
    let commhandler_channels = (gui_update_send, gui_cmd_recv);

    let mut comm_handler = CommHandler::new(threads,
                                             downloadthreads_data.to_downloads(),
                                            //Vec::new(),
                                            commhandler_channels);

    thread::spawn(move || {
        loop {
            comm_handler.update();
            thread::sleep(Duration::from_millis(0));
        }
    });

    // start gtk gui
    gui::gui(parsed_data, gui_update_recv, gui_cmd_send);
}
