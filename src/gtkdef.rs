use glib;
use gtk_sys;
use gdk;
use gobject_sys;
use libc::ssize_t;
use gtk_sys::{GtkStyleProvider, GtkCssProvider};
use gtk::{CssProvider, StyleContext, is_initialized};
use glib::translate::ToGlibPtr;

pub trait RawCssLoad {
    fn load_from_data(data: &str) -> Result<CssProvider, glib::Error>;
}

impl RawCssLoad for CssProvider {
    fn load_from_data(data: &str) -> Result<CssProvider, glib::Error> {
        if !is_initialized() {
            panic!("Gtk not initialized");
        }
        unsafe {
            let pointer = gtk_sys::gtk_css_provider_new();
            let mut error = ::std::ptr::null_mut();
            gtk_sys::gtk_css_provider_load_from_data(pointer,
                                                     data.as_ptr() as *mut u8,
                                                     data.len() as ssize_t,
                                                     &mut error);
            if error.is_null() {
                let translated: CssProvider = glib::translate::from_glib_full(pointer);
                Ok(translated)
            } else {
                Err(glib::Error::wrap(error))
            }
        }
    }
}

pub trait AddCssProvider {
    fn add_provider_for_screen(screen: &gdk::Screen, provider: &CssProvider, priority: u32);
}

impl AddCssProvider for StyleContext {
    fn add_provider_for_screen(screen: &gdk::Screen, provider: &CssProvider, priority: u32) {
        if !is_initialized() {
            panic!("Gtk not initialized");
        }
        unsafe {
            let provider_pointer: *mut GtkCssProvider = provider.to_glib_full();
            let cast_provider_pointer = provider_pointer as *mut GtkStyleProvider;
            gtk_sys::gtk_style_context_add_provider_for_screen(screen.to_glib_none().0,
                                                               cast_provider_pointer,
                                                               priority)
        }
    }
}
