use data::*;
use gtk;
use gtk::prelude::*;
use gtk::{Orientation, Value};
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use glib::types::Type;
use glib::translate::ToGlibPtr;
use helper::*;
use cellrenderers::*;
use theme::*;

// DownloadUpdate {
// Message(String),
// Amount(usize),
// }

// TpoolProgressMsg = (u64, DownloadUpdate);
// GuiCmdMsg = (String, Option<u64>);
// TpoolCmdMsg = GuiCmdMsg;

pub fn gui(data: &mut Vec<Category>,
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
            println!("Failed to get current exe path: {}", e);
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

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("Programming Book Downloader v1.0");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1000, 500);

    // placeholder values
    for category in data.iter_mut() {
        category.set_enable_state_all(true);
        category.begin_downloading_all();
    }

    let downloads = data.to_downloads();
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

    let categoryview = gtk::TreeView::new();
    let category_column_types = [Type::String, Type::Bool];
    let category_store = gtk::TreeStore::new(&category_column_types);
    category_store.add_categories(&data);
    categoryview.add_text_renderer_column("Categories", true, true, true, AddMode::PackStart, 0);
    let toggle_cell = categoryview.add_toggle_renderer_column("Download?",
                                                              false,
                                                              false,
                                                              false,
                                                              AddMode::PackEnd,
                                                              1);
    categoryview.set_model(Some(&category_store));

    // on toggle
    {
        let data = data.clone();
        let command_send_channel = command_send_channel.clone();
        toggle_cell.connect_toggled(move |_, path| {
            // First send message, then update visually - more informative
            let indices = path.get_indices();
            let ref category = data[indices[0] as usize];
            // Update data and the view
            match path.get_depth() {
                1 => {
                    let category = category.to_owned();
                    for download in category.get_downloads().iter() {
                        if let Err(error) = update_download(command_send_channel.clone(),
                                                            download.to_owned()) {
                            println!("{}", error);
                        }
                    }
                }
                2 => {
                    let download = category.get_download_at_idx(indices[1] as usize);
                    if let Err(error) = update_download(command_send_channel.clone(),
                                                        download.to_owned()) {
                        println!("{}", error);
                    }
                }
                _ => {}
            }
            let iter = category_store.get_iter(&path).expect("Invalid TreePath");
            let current_value = category_store.get_value(&iter, 1)
                                              .get::<bool>()
                                              .expect("No Value");
            let new_value = Value::from(!current_value);
            category_store.set_value(&iter, 1, &new_value);
        });
        // treepath references the main list of categories ->
        // if depth == 1 then get list of downloads from the category and send messages with all the
        // hashes
        // if depth == 2 then just send the hash of the individual download
    }

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

    {
        let command_send_channel = command_send_channel.clone();
        window.connect_delete_event(move |_, _| {
            match command_send_channel.clone().send_gui_cmd("stop".to_owned(), None) {
                Ok(_) => {}
                Err(e) => println!("{:?}", e),
            }
            gtk::main_quit();
            Inhibit(false)
        });
    }

    window.show_all();
    gtk::main();
}

// result should be displayed in status bar if error
fn update_download(sender: Sender<GuiCmdMsg>, download: Download) -> Result<(), String> {
    let id = download.get_id();
    let message;
    if download.is_enabled() {
        message = "remove";
    } else {
        message = "add";
    }
    sender.send_gui_cmd(message.to_owned(), Some(id))
}

trait CmdSend {
    fn send_gui_cmd(&self, cmd: String, id: Option<u64>) -> Result<(), String>;
}

impl CmdSend for Sender<GuiCmdMsg> {
    fn send_gui_cmd(&self, cmd: String, id: Option<u64>) -> Result<(), String> {
        match self.send((cmd, id)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

trait AddCategories {
    fn add_category(&self, category: &Category);
    fn add_categories(&self, categories: &[Category]);
}

impl AddCategories for gtk::TreeStore {
    fn add_category(&self, category: &Category) {
        let category_name = category.get_name();
        let downloads = category.get_downloads();
        let iter = self.append(None);
        let category_download_bool = Value::from(category.is_enabled());
        self.set_value(&iter, 0, &Value::from(category_name));
        self.set_value(&iter, 1, &category_download_bool);
        // add all of the downloads
        for download in downloads.iter() {
            let download_name = download.get_name();
            let download_download_bool = Value::from(download.is_enabled());
            let child_iter = self.append(Some(&iter));
            self.set_value(&child_iter, 0, &Value::from(download_name));
            self.set_value(&iter, 1, &download_download_bool);
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
        let iter = self.append();
        self.set_value(&iter, 0, &Value::from(download.0));
        self.set_value(&iter, 1, &Value::from(download.1));
        self.set_value(&iter, 2, &Value::from(download.2));
        self.set_value(&iter, 3, &Value::from(download.3));
        self.set_value(&iter, 4, &Value::from(download.4));
    }
}

// name, size, progress, speed, eta
fn initial_liststore_model(data: &Vec<Download>)
                           -> HashMap<u64, (String, String, f32, String, String)> {
    let mut items = HashMap::new();
    for dl in data.iter() {
        match dl.get_dlinfo() {
            &Some(ref dlinfo) => {
                let dlid = dl.get_id();
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
