extern crate hyper;

use hyper::client::Client;

fn main() {
    if cfg!(windows) {
        let download_link;
        if cfg!(target_pointer_width = "32") {
            // 32 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib32.7z";
        } else {
            // 64 bit
            download_link = "https://github.com/honorabrutroll/mingw-gtk/raw/master/lib64.7z"
        }
        let client = Client::new();
    }
}
