use download::*;
use gtk;
use gdk;
use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::{CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gdk::screen::Screen;
use std::path::{Path, PathBuf};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver};

#[cfg(windows)]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "..\\gtk.css";

#[cfg(not(windows))]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "../gtk.css";

const SECONDARY_GTK_CSS_CONFIG: &'static str = "gtk.css";
const GTK_THEME_CFG: &'static str = "theme.txt";

pub fn gui(data: Vec<Category>,
           update_recv_channel: Receiver<Vec<Category>>,
           command_send_channel: Sender<String>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let current_exe_path;
    match env::current_exe() {
        Ok(exe_path) => current_exe_path = exe_path.clone(),
        Err(e) => {
            println!("failed to get current exe path: {}", e);
            current_exe_path = Path::new("./pbook-gui").to_path_buf()
        }
    };
    let current_working_dir = current_exe_path.parent()
                                              .unwrap_or(Path::new(".."));
    let default_config_path = current_working_dir.join(DEFAULT_GTK_CSS_CONFIG);
    let secondary_config_path = current_working_dir.join(SECONDARY_GTK_CSS_CONFIG);

    setup_theme(current_working_dir,
                default_config_path,
                secondary_config_path);

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

fn setup_theme(current_working_dir: &Path,
               default_config_path: PathBuf,
               secondary_config_path: PathBuf) {
    match get_theme_dir(&current_working_dir) {
        Ok(theme_dir) => {
            // check if gtk css config exists, if so use it
            let mut gtk_theme = String::new();
            match File::open(current_working_dir.join("..").join(GTK_THEME_CFG)) {
                Ok(ref mut gtk_cfg_file) => {
                    match gtk_cfg_file.read_to_string(&mut gtk_theme) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Could not read theme.txt file to string, defaulting to Arc \
                                      theme.");
                            gtk_theme = "arc".to_string();
                        }
                    }
                }
                Err(_) => {
                    match File::open(current_working_dir.join(GTK_THEME_CFG)) {
                        Ok(ref mut gtk_cfg_file) => {
                            match gtk_cfg_file.read_to_string(&mut gtk_theme) {
                                Ok(_) => {}
                                Err(_) => {
                                    println!("Could not read theme.txt file to string, \
                                              defaulting to Arc theme.");
                                    gtk_theme = "arc".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            println!("No theme.txt file found, defaulting to Arc theme.");
                            gtk_theme = "arc".to_string();
                        }
                    }
                }
            }
            let gtk_theme_path = theme_dir.join(gtk_theme).join("gtk.css");
            if default_config_path.exists() {
                let new_css = get_gtk_css(&default_config_path, &gtk_theme_path);
                if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
                    if let Some(screen) = Screen::get_default() {
                        StyleContext::add_provider_for_screen(&screen,
                                                              &style_provider,
                                                              STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
                    } else {
                        println!("here1");
                        no_css_error();
                    }
                } else {
                    println!("here2");
                    no_css_error();
                }
            } else if secondary_config_path.exists() {
                let new_css = get_gtk_css(&secondary_config_path, &gtk_theme_path);
                if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
                    if let Some(screen) = Screen::get_default() {
                        StyleContext::add_provider_for_screen(&screen,
                                                              &style_provider,
                                                              STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
                    } else {
                        println!("here1");
                        no_css_error();
                    }
                } else {
                    println!("here2");
                    no_css_error();
                }
            } else {
                no_css_error();
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn get_gtk_css(config_path: &Path, gtk_theme_path: &Path) -> String {
    let mut gtk_config = String::new();
    match File::open(config_path) {
        Ok(ref mut f) => {
            match f.read_to_string(&mut gtk_config) {
                Ok(_) => {}
                Err(_) => {
                    println!("Could not read gtk config to string, going with defaults");
                    gtk_config = "".to_string();
                }
            }
        }
        Err(_) => {
            println!("Could not open gtk config");
            gtk_config = "".to_string();
        }
    }
    [gtk_config, make_import_css(gtk_theme_path)].join("\n")
}

fn make_import_css(path: &Path) -> String {
    ["@import url(\"", double_slashes(path.to_str().unwrap()).as_str(), "\");\n"].join("")
}

fn double_slashes(p: &str) -> String {
    p.replace("\\", "\\\\")
}

fn get_theme_dir(cwd: &Path) -> Result<PathBuf, String> {
    let first_choice = cwd.join("..").join("themes");
    let second_choice = cwd.join("themes");
    if first_choice.exists() {
        Ok(first_choice)
    } else if second_choice.exists() {
        Ok(second_choice)
    } else {
        Err("Failed to get theme dir".to_string())
    }
}

fn no_css_error() {
    println!("No valid GTK CSS config or gdk screen found, using gtk defaults.");
}
