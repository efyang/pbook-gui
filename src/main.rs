extern crate hyper;
#[macro_use]
extern crate kiss_ui;

use kiss_ui::container::Horizontal;
use kiss_ui::dialog::Dialog;
use kiss_ui::text::Label;

mod download;
mod include;
mod gui;
mod parse;

use include::RAW_DATA;

use std::env;

fn main() {
    println!("Hello, world!");

    match env::current_exe() {
        Ok(exe_path) => println!("Path of this executable is: {}",
                                  exe_path.display()),
        Err(e) => println!("failed to get current exe path: {}", e),
    };
    
    for s in parse::parse(RAW_DATA) {
        println!("{:?}", s);
    }

    kiss_ui::show_gui(|| {
        Dialog::new(
            Horizontal::new(
                children![
                    Label::new("Hello, world!"),
                ]
            )
        )
        .set_title("Hello, world!")
        .set_size_pixels(640, 480)
    });
}
