extern crate hyper;
#[macro_use]
extern crate kiss_ui;

use kiss_ui::container::Horizontal;
use kiss_ui::dialog::Dialog;
use kiss_ui::text::Label;

fn main() {
    println!("Hello, world!");
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
