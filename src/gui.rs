use data::*;
use gtk;
use gtk::prelude::*;
use gtk::{Orientation, Value};
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::collections::HashMap;
use std::cell::RefCell;
use glib;
use glib::types::Type;
use glib::translate::ToGlibPtr;
use helper::*;
use cellrenderers::*;
use theme::*;
use constants::{DEFAULT_GTK_CSS_CONFIG, SECONDARY_GTK_CSS_CONFIG};

// DownloadUpdate {
// Message(String),
// Amount(usize),
// }

// TpoolProgressMsg = (u64, DownloadUpdate);
// GuiCmdMsg = (String, Option<u64>);
// TpoolCmdMsg = GuiCmdMsg;

pub fn gui(data: &mut Vec<Category>,
           update_recv_channel: Receiver<GuiUpdateMsg>,
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
    //for category in data.iter_mut() {
        //category.set_enable_state_all(true);
        //category.begin_downloading_all();
    //}

    *DOWNLOADS.lock().unwrap() = Vec::new();
    let initial_model = make_liststore_model(&*DOWNLOADS.lock().unwrap());
    *ID_DOWNLOAD_HM.lock().unwrap() = {
        let mut id_download_hm = HashMap::new();
        for download in DOWNLOADS.lock().unwrap().iter() {
            id_download_hm.insert(download.get_id(), download.clone());
        }
        id_download_hm
    };
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

    for item in initial_model {
        download_store.add_download(item.1);
    }

    downloadview.set_model(Some(&download_store));
    downloadview.set_headers_visible(true);

    // Setup TLS
    GTK_GLOBAL.with(move |gtk_global| {
        *gtk_global.borrow_mut() = Some((download_store, update_recv_channel));
    });

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
    let toggle_cell = categoryview.add_toggle_renderer_column("Enabled?",
                                                              false,
                                                              false,
                                                              false,
                                                              AddMode::PackEnd,
                                                              1);
    categoryview.set_model(Some(&category_store));

    // on toggle
    {
        let data = data.to_owned();
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
            let new_value = (!current_value).to_value();
            category_store.set_value(&iter, 1, &new_value);
            // make this set all of a category somehow
        });
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

lazy_static! {
    static ref DOWNLOADS: Mutex<Vec<Download>> = Mutex::new(Vec::new());
    static ref ID_DOWNLOAD_HM: Mutex<HashMap<u64, Download>> = Mutex::new(HashMap::new());
}

// Threadlocal storage of Gtk Stuff
thread_local!(
    // (main data, download store, message receiver)
    static GTK_GLOBAL: RefCell<Option<(gtk::ListStore, Receiver<GuiUpdateMsg>)>> = RefCell::new(None)
);

// update TLS
fn update_local() -> Continue {
    GTK_GLOBAL.with(|gtk_global| {
        if let Some((ref download_store, ref rx)) = *gtk_global.borrow() {
            if let Ok(changes) = rx.try_recv() {
                // clear and repopulate takes far too long
                // for every change made in commhandler, append to change list
                // send that change list
                // go through change list and update accordingly
                // command, optional id, optional index, optional new value
                // [string, u64, usize, Download]
                for change in changes.iter() {
                    match &change.0 as &str {
                        "remove" => {
                            // remove index
                            let idx = change.2.unwrap();
                            let mut iter = download_store.iter_nth_child(None, idx as i32).expect("failed2");
                            download_store.remove(&mut iter);
                            DOWNLOADS.lock().unwrap().remove(idx);
                        },
                        "add" => {
                            let mut download = change.clone().3.unwrap();
                            download.start_download();
                            download.set_enable_state(true);
                            // add download
                            let values = download_to_values(&download).unwrap().1;
                            download_store.add_download(values);
                            DOWNLOADS.lock().unwrap().push(download);
                        },
                        "set" => {
                            let iter = download_store.iter_nth_child(None, change.2.unwrap() as i32).unwrap();
                            let values = download_to_values(&change.clone().3.unwrap()).unwrap().1;
                            download_store.set_download(&iter, values);
                        },
                        _ => {}
                    }
                }
            }
        }
    });
    Continue(false)
}

pub fn update_gui() {
    if gtk::is_initialized() {
        glib::idle_add(update_local);
    }
}

// result should be displayed in status bar if error
fn update_download(sender: Sender<GuiCmdMsg>, download: Download) -> Result<(), String> {
    let id = download.get_id();
    let message;
    for dl in DOWNLOADS.lock().unwrap().iter() {
        if dl.get_id() == id {
            if dl.is_enabled() {
                message = "remove";
            } else {
                message = "add";
            }
            return sender.send_gui_cmd(message.to_owned(), Some(id));
        }
    }
    // not found in current list
    return sender.send_gui_cmd("add".to_owned(), Some(id));
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
        let category_download_bool = category.is_enabled().to_value();
        self.set_value(&iter, 0, &category_name.to_value());
        self.set_value(&iter, 1, &category_download_bool);
        // add all of the downloads
        for download in downloads.iter() {
            let download_name = download.get_name();
            let download_download_bool = download.is_enabled().to_value();
            let child_iter = self.append(Some(&iter));
            self.set_value(&child_iter, 0, &download_name.to_value());
            self.set_value(&child_iter, 1, &download_download_bool);
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
    fn set_download(&self, iter: &gtk::TreeIter, download: (String, String, f32, String, String));
}

impl AddDownload for gtk::ListStore {
    fn add_download(&self, download: (String, String, f32, String, String)) {
        let iter = self.append();
        self.set_download(&iter, download);
    }
    fn set_download(&self, iter: &gtk::TreeIter, download: (String, String, f32, String, String)) {
        self.set_value(&iter, 0, &download.0.to_value());
        self.set_value(&iter, 1, &download.1.to_value());
        self.set_value(&iter, 2, &download.2.to_value());
        self.set_value(&iter, 3, &download.3.to_value());
        self.set_value(&iter, 4, &download.4.to_value());
    }
}

// name, size, progress, speed, eta
fn make_liststore_model(data: &Vec<Download>)
                           -> HashMap<u64, (String, String, f32, String, String)> {
    let mut items = HashMap::new();
    for dl in data.iter() {
        match download_to_values(dl) {
            Some(values) => {
                items.insert(values.0, values.1);
            }
            None => {}
        }
    }
    items
}

fn download_to_values(dl: &Download) -> Option<(u64, (String, String, f32, String, String))> {
    match dl.get_dlinfo() {
        &Some(ref dlinfo) => {
            let dlid = dl.get_id();
            // shorten needed until ellipsize is implemented for CellRendererText
            let name = dl.get_name().to_string().shorten(50);
            let size = (dlinfo.get_total() as f32).convert_to_byte_units(0);
            let percent = dlinfo.get_percentage();
            let speed = format!("{}/s", dlinfo.get_speed().convert_to_byte_units(0));
            let eta = dlinfo.get_eta();
            Some((dlid, (name, size, percent, speed, eta)))
        }
        &None => None,
    }
}
