use data::*;
use gtk;
use gdk;
use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::{Orientation, CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gdk::screen::Screen;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::iter;

#[cfg(windows)]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "..\\gtk.css";

#[cfg(not(windows))]
const DEFAULT_GTK_CSS_CONFIG: &'static str = "../gtk.css";

const SECONDARY_GTK_CSS_CONFIG: &'static str = "gtk.css";
const GTK_THEME_CFG: &'static str = "theme.txt";

pub fn gui(data: Vec<Category>,
           update_recv_channel: Receiver<Vec<Download>>,
           command_send_channel: Sender<(String, Option<u64>)>) {
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

    window.set_title("Programming Book Downloader v1.0");
    window.set_border_width(10);
    window.set_window_position(gtk::WindowPosition::Center);
    window.set_default_size(900, 700);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    let mut downloads = data.to_downloads();
    for download in downloads.iter_mut() {
        download.start_download();
    }
    let mut infoitems = initial_render(&downloads);

    let button = gtk::Button::new_with_label("Click me!").unwrap();
    let listbox = gtk::ListBox::new().unwrap();

    for item in infoitems {
        println!("{:?}", item);
    }

    let scroll = gtk::ScrolledWindow::new(None, None).unwrap();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add(&listbox);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10).unwrap();
    vbox.pack_start(&scroll, true, true, 0);
    vbox.pack_end(&button, false, true, 0);

    // holds both the category list and the info list
    let lists_holder = gtk::Box::new(gtk::Orientation::Horizontal, 0).unwrap();
    lists_holder.pack_end(&vbox, true, true, 0);
    window.add(&lists_holder);

    window.show_all();
    gtk::main();

}
//
//name, size, progress, speed, eta
fn initial_render(data: &Vec<Download>) -> HashMap<u64, (String, String, f32, String, String)> {
    let mut items = HashMap::new();
    for dl in data.iter() {
        match dl.get_dlinfo() {
            &Some(ref dlinfo) => {
                let dlid = dl.id();
                let name = dl.get_name().to_string();
                let size = (dlinfo.get_total() as f32).convert_to_byte_units(0);
                let percent = dlinfo.get_percentage();
                let speed = format!("{}/s", dlinfo.get_speed().convert_to_byte_units(0));
                let eta = dlinfo.get_eta().to_string();
                items.insert(dlid, (name, size, percent, speed, eta));
            },
            &None => {},
        }
    }
    items
}

const BYTE_UNITS: [(f32, &'static str); 4] = [(0.0, "B"), (1024.0, "KiB"), (1048576.0, "MiB"), (1073741800.0, "GiB")];

trait ToByteUnits {
    fn convert_to_byte_units(&self, decimal_places: usize) -> String;
}

impl ToByteUnits for f32 {
    fn convert_to_byte_units(&self, decimal_places: usize) -> String {
        let mut bunit = (0.0f32, "B");
        for bidx in 0..BYTE_UNITS.len() - 1 {
            let min = BYTE_UNITS[bidx];
            let max = BYTE_UNITS[bidx + 1];
            if (self >= &min.0) && (self < &max.0) {
                bunit = min;
            }
        }
        let last = BYTE_UNITS.last().unwrap().clone();
        if self >= &last.0 {
            bunit = last;
        }
        let divided = self/bunit.0 as f32;
        format!("{}{}", round_to_places(divided, decimal_places), bunit.1)
    }
}

trait Repetition {
    fn repeat(&self, times: usize) -> String;
}

impl Repetition for str {
    fn repeat(&self, times: usize) -> String {
        iter::repeat(self).take(times).map(|s| s.clone()).collect::<String>()
    }
}

// places refers to places after decimal point
fn round_to_places(n: f32, places: usize) -> f32 {
    let div = (format!("1{}", &"0".repeat(places))).parse::<f32>().unwrap();
    (n * div).round() / div
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
                        println!("default success");
                    } else {
                        println!("here1 default");
                        no_css_error();
                    }
                } else {
                    println!("here2 default");
                    no_css_error();
                }
            } else if secondary_config_path.exists() {
                let new_css = get_gtk_css(&secondary_config_path, &gtk_theme_path);
                if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
                    if let Some(screen) = Screen::get_default() {
                        StyleContext::add_provider_for_screen(&screen,
                                                              &style_provider,
                                                              STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
                        println!("secondary success");
                    } else {
                        println!("here1 secondary");
                        no_css_error();
                    }
                } else {
                    CssProvider::load_from_data(&new_css).expect("println");
                    println!("here2 secondary");
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
