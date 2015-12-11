#![allow(dead_code, unused_imports, unused_attributes)]
extern crate hyper;
#[macro_use]
extern crate gtk;

mod download;
mod include;
mod gui;
mod parse;

use include::RAW_DATA;

use gtk::traits::*;
use gtk::signal::Inhibit;
use std::env;

fn main() {
    match env::current_exe() {
        Ok(exe_path) => println!("Path of this executable is: {}", exe_path.display()),
        Err(e) => println!("failed to get current exe path: {}", e),
    };

    for s in parse::parse(RAW_DATA) {
        // println!("{:?}", s);
        match parse::get_item_info(s) {
            Some(info) => {
                let dl = download::Download::new(info.0, info.1);
                println!("{:?}", dl);
            }
            None => {}
        }
    }
    // start gtk
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();

    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_window_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let button = gtk::Button::new_with_label("Click me!").unwrap();

    window.add(&button);

    window.show_all();
    gtk::main();
}
