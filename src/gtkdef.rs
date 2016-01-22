use gtk;
use glib;
use gtk_sys;
use libc::ssize_t;
use gtk::CssProvider;
use std::ptr;

trait RawCssLoad {
    fn load_from_data(data: &str) -> Result<CssProvider, glib::Error>;
}

impl RawCssLoad for CssProvider {
    fn load_from_data(data: &str) -> Result<CssProvider, glib::Error> {
        // This must be run only after the main thread is initialized
        unsafe {
            let pointer = gtk_sys::gtk_css_provider_new();
            let mut error = ::std::ptr::null_mut();
            gtk_sys::gtk_css_provider_load_from_data(pointer, data.as_ptr() as *mut u8, data.len() as ssize_t, &mut error);
            if error.is_null() {
                Ok(glib::translate::from_glib_full(pointer))
            } else {
                Err(glib::Error::wrap(error))
            }
        }
    }
}
