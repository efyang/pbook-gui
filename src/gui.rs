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
    let mut infoboxes = initial_render(&downloads);

    let button = gtk::Button::new_with_label("Click me!").unwrap();
    let listbox = gtk::ListBox::new().unwrap();

    //let mut labels = Vec::new();
    for infobox in infoboxes.values() {
        //labels.push(gtk::Label::new(&("label ".to_string() + &i.to_string())).unwrap());
        listbox.add(infobox);
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

fn initial_render(data: &Vec<Download>) -> HashMap<u64, gtk::Box> {
    let mut boxes = HashMap::new();
    for dl in data.iter() {
        match dl.get_dlinfo() {
            &Some(ref dlinfo) => {
                let dlid = dl.id();
                let bar = make_bar(dlinfo);
                let infobox = gtk::Box::new(Orientation::Horizontal, 0).unwrap();
                let namelabel = make_name_label(&truncate_str(dl.get_name(), 90));
                infobox.pack_start(&namelabel, true, true, 0);
                infobox.add(&bar);
                boxes.insert(dlid, infobox);
            },
            &None => {},
        }
    }
    boxes
}

fn truncate_str(s: &str, maxchars: usize) -> String {
    if s.len() <= maxchars {
        s.to_string()
    } else {
        s[0..maxchars - 3].to_string() + "..."
    }
}

fn make_bar(dlinfo: &DownloadInfo) -> gtk::ProgressBar {
    let mut bar = gtk::ProgressBar::new().unwrap();
    let percent = dlinfo.get_progress() as f64/dlinfo.get_total() as f64;
    bar.set_fraction(percent);
    //&mut bar as *mut gtk::ProgressBar
    bar
}

fn make_download_speed_label(dlinfo: &DownloadInfo) -> gtk::Label {
    unimplemented!();
}

fn make_name_label(name: &str) -> gtk::Label {
    let namelabel = gtk::Label::new(name).unwrap();
    namelabel.set_halign(gtk::Align::Start);
    namelabel
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
