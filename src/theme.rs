use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

use gdk::Screen;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, StyleContext};

use gtkdef::*;
use constants::GTK_THEME_CFG;
// Manually implemented with a trait until its implemented in the main branch
pub fn setup_theme(current_working_dir: &Path,
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
                            gtk_theme = "arc".to_owned();
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
                                    gtk_theme = "arc".to_owned();
                                }
                            }
                        }
                        Err(_) => {
                            println!("No theme.txt file found, defaulting to Arc theme.");
                            gtk_theme = "arc".to_owned();
                        }
                    }
                }
            }
            gtk_theme = gtk_theme.trim().to_owned();
            let gtk_theme_path = theme_dir.join(gtk_theme).join("gtk.css");
            if default_config_path.exists() {
                let new_css = get_gtk_css(&default_config_path, &gtk_theme_path);
                if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
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
            } else if secondary_config_path.exists() {
                let new_css = get_gtk_css(&secondary_config_path, &gtk_theme_path);
                if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
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
                    gtk_config = "".to_owned();
                }
            }
        }
        Err(_) => {
            println!("Could not open gtk config");
            gtk_config = "".to_owned();
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
        Err("Failed to get theme dir".to_owned())
    }
}

fn no_css_error() {
    println!("No valid GTK CSS config or gdk screen found, using gtk defaults.");
}
