use gdk::{EventButton, EventType};
use gdk::enums::modifier_type::Button3Mask;

pub fn is_right_click(ebutton: &EventButton) -> bool {
    if let EventType::ButtonRelease = ebutton.get_event_type() {
        let modtype = ebutton.get_state();
        modtype.contains(Button3Mask)
    } else {
        false
    }
}
