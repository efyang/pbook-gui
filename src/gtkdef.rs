use glib;
use gtk_sys;
use gdk;
use gobject_sys::g_object_set;
use pango_sys::PangoEllipsizeMode;
use libc::{ssize_t, c_void, c_char};
use gtk_sys::{GtkStyleProvider, GtkCssProvider};
use gtk::{CssProvider, StyleContext, is_initialized, CellRendererText, ToValue};
use glib::translate::{ToGlibPtr, Stash};
use std::ffi::CString;

pub trait SetEllipsizeMode {
    fn set_ellipsize_mode(&self, mode: PangoEllipsizeMode);
}

impl SetEllipsizeMode for CellRendererText {
    fn set_ellipsize_mode(&self, mode: PangoEllipsizeMode) {
        if !is_initialized() {
            panic!("Gtk not initialized");
        }
        let stash: Stash<*mut gtk_sys::GtkCellRendererText, _> = self.to_glib_none();
        let pointer = stash.0 as *mut c_void;
        let nullptr: *const c_void = ::std::ptr::null();
        let ell_set_ptr: *const c_char = CString::new("ellipsize-set").unwrap().as_ptr();
        let ell_ptr: *const c_char = CString::new("ellipsize").unwrap().as_ptr();
        // set "ellipsize-set" to true
        unsafe {
            g_object_set(pointer, ell_set_ptr, true.to_value(), nullptr);

        }
        // set "ellipsize" to mode
        unsafe {
            g_object_set(pointer, ell_ptr, mode, nullptr);
        }
    }
}

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
