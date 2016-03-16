extern crate hyper;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate num_cpus;
extern crate threadpool;
extern crate time;
extern crate gtk_sys;
extern crate gobject_sys;
extern crate pango_sys;
extern crate gdk_pixbuf;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;

mod data;
mod include;
mod gui;
mod parse;
mod commhandler;
mod downloader;
mod helper;
mod gtkdef;
mod cellrenderers;
mod theme;
mod constants;
#[allow(unused_variables, unused_imports, dead_code)]
mod fsthread;

use commhandler::*;
use parse::*;
use include::RAW_DATA;

fn main() {
    let threads = num_cpus::get();
    let mut parsed_data: Vec<Category> = parse(RAW_DATA);
    let downloadthreads_data = parsed_data.clone();

    // initialize the channels between gui and comm handler
    let (gui_update_send, gui_update_recv) = channel::<GuiUpdateMsg>();
    let (gui_cmd_send, gui_cmd_recv) = channel::<GuiCmdMsg>();
    let commhandler_channels = (gui_update_send, gui_cmd_recv);

    let mut comm_handler = CommHandler::new(threads,
                                            downloadthreads_data.to_downloads(),
                                            // Vec::new(),
                                            commhandler_channels);

    thread::spawn(move || {
        loop {
            comm_handler.update();
            thread::sleep(Duration::new(0, 30));
        }
    });

    // start gtk gui
    gui::gui(&mut parsed_data, gui_update_recv, gui_cmd_send);
}
