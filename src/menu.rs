use gtk_sys::gtk_menu_popup;
use gtk::Menu;
use std::ptr::null_mut;
use libc::c_uint;
use glib::translate::ToGlibPtr;

pub trait Popup {
    fn popup(&self, button: u32, activate_time: u32);
}

impl Popup for Menu {
    fn popup(&self, button: u32, activate_time: u32) {
        unsafe {
            gtk_menu_popup(self.to_glib_none().0, null_mut(), null_mut(), None, null_mut(), button as c_uint, activate_time);
        }
    }
}
