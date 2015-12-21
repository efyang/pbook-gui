use download::*;
use gtk;
use gdk;
use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::{CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gdk::screen::Screen;
use std::path::Path;
use std::env;

#[cfg(windows)]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "..\\gtk.css";

#[cfg(not(windows))]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "../gtk.css";

const SECONDARY_GTK_CSS_CONFIG: &'static str = "gtk.css";

pub fn gui(data: Vec<Category>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let current_exe_path;
    match env::current_exe() {
        Ok(exe_path) => current_exe_path = exe_path.clone(),
        Err(e) => {
            println!("failed to get current exe path: {}", e);
            current_exe_path = Path::new("./placeholder").to_path_buf()
        }
    };
    let current_working_dir = current_exe_path.parent().unwrap_or(Path::new("./"));

    // check if gtk css config exists, if so use it
    if current_working_dir.join(DEFAULT_GTK_CSS_CONFIG).exists() {
        if let Ok(style_provider) =
               CssProvider::load_from_path(current_working_dir.join(DEFAULT_GTK_CSS_CONFIG)
                                                              .to_str()
                                                              .unwrap()) {
            if let Some(screen) = Screen::get_default() {
                StyleContext::add_provider_for_screen(&screen,
                                                      &style_provider,
                                                      STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
            } else {
                no_css_error();
            }
        } else {
            no_css_error();
        }
    } else {
        if current_working_dir.join(SECONDARY_GTK_CSS_CONFIG).exists() {
            if let Ok(style_provider) =
                   CssProvider::load_from_path(current_working_dir.join(SECONDARY_GTK_CSS_CONFIG)
                                                                  .to_str()
                                                                  .unwrap()) {
                if let Some(screen) = Screen::get_default() {
                    StyleContext::add_provider_for_screen(&screen,
                                                          &style_provider,
                                                          STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
                } else {
                    no_css_error();
                }
            } else {
                no_css_error();
            }
        } else {
            no_css_error();
        }
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();

    window.set_title("Listbox Testing");
    window.set_border_width(10);
    window.set_window_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 500);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let button = gtk::Button::new_with_label("Click me!").unwrap();
    let listbox = gtk::ListBox::new().unwrap();

    let mut labels = Vec::new();
    for i in 0..100 {
        labels.push(gtk::Label::new(&("label ".to_string() + &i.to_string())).unwrap());
        listbox.add(&labels[i]);
    }

    let scroll = gtk::ScrolledWindow::new(None, None).unwrap();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add(&listbox);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 30).unwrap();
    vbox.pack_start(&scroll, true, true, 0);
    vbox.pack_end(&button, false, true, 0);

    window.add(&vbox);

    window.show_all();
    gtk::main();

}

fn no_css_error() {
    println!("No valid GTK CSS config or gdk screen found, using gtk defaults.");
}
