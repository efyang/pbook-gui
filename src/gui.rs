use data::*;
use gtk;
use gdk;
use gtk::prelude::*;
use gtk::{Orientation, CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION, IsA,
          CellRenderer};
use gdk::screen::Screen;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use glib::{Value, Type};
use helper::*;

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

    // setup_theme(current_working_dir,
    // default_config_path,
    // secondary_config_path);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

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
    let mut infoitems = initial_liststore_model(&downloads);
    // main rendering
    let button = gtk::Button::new_with_label("Click me!");

    let downloadview = gtk::TreeView::new();
    // name, size, progress, speed, eta
    let download_column_types = [Type::String, Type::String, Type::F32, Type::String, Type::String];
    let download_store = gtk::ListStore::new(&download_column_types);
    downloadview.add_text_renderer_column("Name", true, true, false, AddMode::PackStart, 0);
    downloadview.add_text_renderer_column("Size", true, true, false, AddMode::PackStart, 1);
    downloadview.add_progress_renderer_column("Progress", true, true, true, AddMode::PackStart, 2);
    downloadview.add_text_renderer_column("Speed", true, true, false, AddMode::PackStart, 3);
    downloadview.add_text_renderer_column("ETA", true, true, false, AddMode::PackStart, 4);

    for item in infoitems {
        download_store.add_download(item.1);
    }

    downloadview.set_model(Some(&download_store));
    downloadview.set_headers_visible(true);
    
    // add the scroll
    let download_scroll = gtk::ScrolledWindow::new(None, None);
    download_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    download_scroll.add(&downloadview);
    
    // put the scroll and downloads together
    let download_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    download_box.pack_start(&download_scroll, true, true, 0);
    download_box.pack_end(&button, false, true, 0);

    // cellrenderertoggle for checkbox in treeview
    let categoryview = gtk::TreeView::new();
    let category_column_types = [Type::String];
    let category_store = gtk::TreeStore::new(&category_column_types);
    category_store.add_categories(&data);
    categoryview.add_text_renderer_column("Categories", true, true, true, AddMode::PackStart, 0);
    categoryview.set_model(Some(&category_store));

    let category_scroll = gtk::ScrolledWindow::new(None, None);
    category_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    category_scroll.add(&categoryview);

    let category_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    category_box.pack_start(&category_scroll, true, true, 0);
    
    // holds both the category list and the info list
    let lists_holder = gtk::Box::new(gtk::Orientation::Horizontal, 30);
    lists_holder.pack_start(&category_box, true, true, 0);
    lists_holder.pack_end(&download_box, true, true, 0);
    window.add(&lists_holder);

    window.show_all();
    gtk::main();

}

trait AddCategories {
    fn add_category(&self, category: &Category);
    fn add_categories(&self, categories: &[Category]);
}

impl AddCategories for gtk::TreeStore {
    fn add_category(&self, category: &Category) {
        let category_name = category.get_name();
        let downloads = category.downloads();
        let iter = self.append(None);
        self.set_string(&iter, 0, category_name);
        // add all of the downloads
        for download in downloads.iter() {
            let download_name = download.get_name();
            let child_iter = self.append(Some(&iter));
            self.set_string(&child_iter, 0, download_name);
        }
    }

    fn add_categories(&self, categories: &[Category]) {
        for category in categories.iter() {
            self.add_category(category);
        }
    }
}

trait AddDownload {
    fn add_download(&self, download: (String, String, f32, String, String));
}

impl AddDownload for gtk::ListStore {
    fn add_download(&self, download: (String, String, f32, String, String)) {
        let progress = unsafe {
            let mut progress;
            progress = Value::new();
            progress.init(Type::F32);
            progress.set(&download.2);
            progress
        };
        let iter = self.append();
        self.set_string(&iter, 0, &download.0);
        self.set_string(&iter, 1, &download.1);
        self.set_value(&iter, 2, &progress);
        self.set_string(&iter, 3, &download.3);
        self.set_string(&iter, 4, &download.4);
    }
}

enum AddMode {
    PackEnd,
    PackStart,
}

trait AddCellRenderers {
    fn add_cell_renderer_column<T: IsA<CellRenderer>>(&self,
                                                      title: &str,
                                                      cell: &T,
                                                      fill: bool,
                                                      resizable: bool,
                                                      expand: bool,
                                                      add_mode: AddMode,
                                                      attribute_type: &str,
                                                      column_number: i32);
    fn add_text_renderer_column(&self,
                                title: &str,
                                fill: bool,
                                resizable: bool,
                                expand: bool,
                                add_mode: AddMode,
                                column_number: i32);
    fn add_progress_renderer_column(&self,
                                    title: &str,
                                    fill: bool,
                                    resizable: bool,
                                    expand: bool,
                                    add_mode: AddMode,
                                    column_number: i32);
}

impl AddCellRenderers for gtk::TreeView {
    fn add_cell_renderer_column<T: IsA<CellRenderer>>(&self,
                                                      title: &str,
                                                      cell: &T,
                                                      fill: bool,
                                                      resizable: bool,
                                                      expand: bool,
                                                      add_mode: AddMode,
                                                      attribute_type: &str,
                                                      column_number: i32) {
        let column = gtk::TreeViewColumn::new();
        match add_mode {
            AddMode::PackEnd => {
                column.pack_end(cell, fill);
            }
            AddMode::PackStart => {
                column.pack_start(cell, fill);
            }
        }
        column.add_attribute(cell, attribute_type, column_number);
        column.set_title(title);
        column.set_resizable(resizable);
        column.set_expand(expand);
        self.append_column(&column);
    }

    fn add_text_renderer_column(&self,
                                title: &str,
                                fill: bool,
                                resizable: bool,
                                expand: bool,
                                add_mode: AddMode,
                                column_number: i32) {
        let cell = gtk::CellRendererText::new();
        self.add_cell_renderer_column(title,
                                      &cell,
                                      fill,
                                      resizable,
                                      expand,
                                      add_mode,
                                      "text",
                                      column_number);
    }

    fn add_progress_renderer_column(&self,
                                    title: &str,
                                    fill: bool,
                                    resizable: bool,
                                    expand: bool,
                                    add_mode: AddMode,
                                    column_number: i32) {
        let cell = gtk::CellRendererProgress::new();
        self.add_cell_renderer_column(title,
                                      &cell,
                                      fill,
                                      resizable,
                                      expand,
                                      add_mode,
                                      "value",
                                      column_number);
    }
}

// name, size, progress, speed, eta
fn initial_liststore_model(data: &Vec<Download>)
                           -> HashMap<u64, (String, String, f32, String, String)> {
    let mut items = HashMap::new();
    for dl in data.iter() {
        match dl.get_dlinfo() {
            &Some(ref dlinfo) => {
                let dlid = dl.id();
                // shorten needed until ellipsize is implemented for CellRendererText
                let name = dl.get_name().to_string().shorten(50);
                let size = (dlinfo.get_total() as f32).convert_to_byte_units(0);
                let percent = dlinfo.get_percentage();
                let speed = format!("{}/s", dlinfo.get_speed().convert_to_byte_units(0));
                let eta = dlinfo.get_eta().to_string();
                items.insert(dlid, (name, size, percent, speed, eta));
            }
            &None => {}
        }
    }
    items
}

fn initial_categorystore_model(data: &Vec<Category>) -> () {
    unimplemented!();
}

trait RawCssLoad {
    fn load_from_data(data: &str) {

    }
}

// TODO: Manually implement with a trait until its implemented in the main branch

// useless until the following are regenned:
// https://github.com/gtk-rs/gtk/blob/master/src/auto/style_context.rs#L36
// https://github.com/gtk-rs/gtk/blob/master/src/auto/css_provider.rs#L26
//
// fn setup_theme(current_working_dir: &Path,
// default_config_path: PathBuf,
// secondary_config_path: PathBuf) {
// match get_theme_dir(&current_working_dir) {
// Ok(theme_dir) => {
// check if gtk css config exists, if so use it
// let mut gtk_theme = String::new();
// match File::open(current_working_dir.join("..").join(GTK_THEME_CFG)) {
// Ok(ref mut gtk_cfg_file) => {
// match gtk_cfg_file.read_to_string(&mut gtk_theme) {
// Ok(_) => {}
// Err(_) => {
// println!("Could not read theme.txt file to string, defaulting to Arc \
// theme.");
// gtk_theme = "arc".to_string();
// }
// }
// }
// Err(_) => {
// match File::open(current_working_dir.join(GTK_THEME_CFG)) {
// Ok(ref mut gtk_cfg_file) => {
// match gtk_cfg_file.read_to_string(&mut gtk_theme) {
// Ok(_) => {}
// Err(_) => {
// println!("Could not read theme.txt file to string, \
// defaulting to Arc theme.");
// gtk_theme = "arc".to_string();
// }
// }
// }
// Err(_) => {
// println!("No theme.txt file found, defaulting to Arc theme.");
// gtk_theme = "arc".to_string();
// }
// }
// }
// }
// let gtk_theme_path = theme_dir.join(gtk_theme).join("gtk.css");
// if default_config_path.exists() {
// let new_css = get_gtk_css(&default_config_path, &gtk_theme_path);
// if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
// if let Some(screen) = Screen::get_default() {
// StyleContext::add_provider_for_screen(&screen,
// &style_provider,
// STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
// println!("default success");
// } else {
// println!("here1 default");
// no_css_error();
// }
// } else {
// println!("here2 default");
// no_css_error();
// }
// } else if secondary_config_path.exists() {
// let new_css = get_gtk_css(&secondary_config_path, &gtk_theme_path);
// if let Ok(style_provider) = CssProvider::load_from_data(&new_css) {
// if let Some(screen) = Screen::get_default() {
// StyleContext::add_provider_for_screen(&screen,
// &style_provider,
// STYLE_PROVIDER_PRIORITY_APPLICATION as u32);
// println!("secondary success");
// } else {
// println!("here1 secondary");
// no_css_error();
// }
// } else {
// CssProvider::load_from_data(&new_css).expect("println");
// println!("here2 secondary");
// no_css_error();
// }
// } else {
// no_css_error();
// }
// }
// Err(e) => {
// println!("{}", e);
// }
// }
// }
//
// fn get_gtk_css(config_path: &Path, gtk_theme_path: &Path) -> String {
// let mut gtk_config = String::new();
// match File::open(config_path) {
// Ok(ref mut f) => {
// match f.read_to_string(&mut gtk_config) {
// Ok(_) => {}
// Err(_) => {
// println!("Could not read gtk config to string, going with defaults");
// gtk_config = "".to_string();
// }
// }
// }
// Err(_) => {
// println!("Could not open gtk config");
// gtk_config = "".to_string();
// }
// }
// [gtk_config, make_import_css(gtk_theme_path)].join("\n")
// }
//
// fn make_import_css(path: &Path) -> String {
// ["@import url(\"", double_slashes(path.to_str().unwrap()).as_str(), "\");\n"].join("")
// }
//
// fn double_slashes(p: &str) -> String {
// p.replace("\\", "\\\\")
// }
//
// fn get_theme_dir(cwd: &Path) -> Result<PathBuf, String> {
// let first_choice = cwd.join("..").join("themes");
// let second_choice = cwd.join("themes");
// if first_choice.exists() {
// Ok(first_choice)
// } else if second_choice.exists() {
// Ok(second_choice)
// } else {
// Err("Failed to get theme dir".to_string())
// }
// }
//
// fn no_css_error() {
// println!("No valid GTK CSS config or gdk screen found, using gtk defaults.");
// }
//
//
